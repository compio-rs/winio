#![allow(non_snake_case)]

use std::{
    collections::HashMap,
    env::set_var,
    fs::File,
    io::{BufRead, BufReader},
    os::{raw::c_void, unix::prelude::*},
    sync::{LazyLock, Mutex, OnceLock},
    thread::spawn,
};

use jni::{
    Env, EnvUnowned, JavaVM,
    errors::{Error, JniError, Result as JniResult},
    objects::JObject,
    sys::{JNI_VERSION_1_6, jint},
};

use crate::GlobalRef;

pub(crate) static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();
static ACTIVITY: Mutex<Option<GlobalRef>> = Mutex::new(None);
static UI_THREAD_PENDINGS: LazyLock<Mutex<HashMap<i64, Box<dyn FnOnce() + Send + Sync>>>> =
    LazyLock::new(Default::default);

#[derive(Debug)]
pub enum ActivityMessage {
    Created,
    Start,
    Resume,
    Pause,
    Stop,
    Destroy,
    ConfigChanged(GlobalRef),
    LowMemory,
    RunUiThread(i64),
}

unsafe extern "system" {
    fn main();
}

fn handle_message(activity: JObject, message: ActivityMessage) {
    match message {
        ActivityMessage::Created => {
            if let Some(vm) = JAVA_VM.get() {
                let _ = vm.attach_current_thread::<_, (), Error>(|env| {
                    if let Ok(mut lock) = ACTIVITY.lock() {
                        lock.replace(env.new_global_ref(activity)?);
                        drop(lock);
                        spawn(|| unsafe { main() });
                    }
                    Ok(())
                });
            }
        }

        ActivityMessage::Destroy => {
            if let Ok(mut lock) = ACTIVITY.lock() {
                lock.take();
            }
        }

        ActivityMessage::RunUiThread(id) => {
            if let Ok(mut lock) = UI_THREAD_PENDINGS.lock()
                && let Some(f) = lock.remove(&id)
            {
                drop(lock);
                f();
            }
        }

        _ => (),
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1start(
    _env: EnvUnowned<'_>,
    activity: JObject,
) {
    handle_message(activity, ActivityMessage::Start);
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1resume(
    _env: EnvUnowned<'_>,
    activity: JObject,
) {
    handle_message(activity, ActivityMessage::Resume);
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1pause(
    _env: EnvUnowned<'_>,
    activity: JObject,
) {
    handle_message(activity, ActivityMessage::Pause);
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1stop(_env: EnvUnowned<'_>, activity: JObject) {
    handle_message(activity, ActivityMessage::Stop);
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1destroy(
    _env: EnvUnowned<'_>,
    activity: JObject,
) {
    handle_message(activity, ActivityMessage::Destroy);
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1configuration_1changed(
    mut env: EnvUnowned<'_>,
    activity: JObject,
    new_config: JObject,
) {
    let _ = env.with_env_no_catch::<_, (), Error>(|env| {
        handle_message(
            activity,
            ActivityMessage::ConfigChanged(env.new_global_ref(new_config)?),
        );
        Ok(())
    });
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1low_1memory(
    _env: EnvUnowned<'_>,
    activity: JObject,
) {
    handle_message(activity, ActivityMessage::LowMemory);
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1create(
    _env: EnvUnowned<'_>,
    activity: JObject,
) {
    handle_message(activity, ActivityMessage::Created)
}

#[unsafe(no_mangle)]
extern "system" fn Java_rs_compio_winio_Activity_on_1run_1ui_1thread(
    _env: EnvUnowned<'_>,
    activity: JObject,
    id: i64,
) {
    handle_message(activity, ActivityMessage::RunUiThread(id));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn JNI_OnLoad(vm: JavaVM, _: *mut c_void) -> jint {
    JAVA_VM.set(vm).unwrap();
    #[cfg(debug_assertions)]
    unsafe {
        set_var("RUST_BACKTRACE", "full");
    }

    let mut log_pipe: [RawFd; 2] = Default::default();
    unsafe {
        libc::pipe(log_pipe.as_mut_ptr());
        libc::dup2(log_pipe[1], libc::STDOUT_FILENO);
        libc::dup2(log_pipe[1], libc::STDERR_FILENO);
    }
    spawn(move || unsafe {
        let file = File::from_raw_fd(log_pipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                }
                log::info!("{}", buffer);
            }
        }
    });

    JNI_VERSION_1_6
}

pub fn vm_exec<F, R>(f: F) -> JniResult<R>
where
    F: FnOnce(&mut Env<'_>, GlobalRef) -> JniResult<R>,
{
    if let Some(vm) = JAVA_VM.get() {
        vm.attach_current_thread::<_, R, Error>(|env| {
            let activity = match ACTIVITY
                .lock()
                .ok()
                .and_then(|lock| lock.as_ref().map(|a| a.as_obj()))
            {
                Some(activity) => env.new_global_ref(activity)?,
                None => GlobalRef::null(),
            };
            f(env, activity)
        })
    } else {
        Err(Error::NullPtr("No java VM."))
    }
}

pub fn vm_exec_on_ui_thread<F, R>(f: F) -> JniResult<R>
where
    F: FnOnce(&mut Env<'_>, GlobalRef) -> JniResult<R> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    static mut IDX: i64 = 0;

    vm_exec(|env, act| {
        let Ok(mut lock) = UI_THREAD_PENDINGS.lock() else {
            return Err(Error::JniCall(JniError::Unknown));
        };

        let (tx, rx) = oneshot::channel();
        unsafe {
            IDX += 1;
            let act2 = env.new_global_ref(act.as_obj())?;
            lock.insert(
                IDX,
                Box::new(move || {
                    let _ = if let Some(vm) = JAVA_VM.get() {
                        tx.send(vm.attach_current_thread::<_, R, Error>(|env| f(env, act2)))
                    } else {
                        tx.send(Err(Error::NullPtr("No java VM.")))
                    };
                }),
            );
            drop(lock);
            env.call_method(
                act.as_obj(),
                jni::jni_str!("runOnUiThread"),
                jni::jni_sig!("(J)V"),
                &[IDX.into()],
            )?;
        }
        rx.recv().unwrap()
    })
}
