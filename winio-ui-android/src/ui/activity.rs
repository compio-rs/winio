#![allow(non_snake_case)]

use {
    jni::{
        AttachGuard, JNIEnv, JavaVM,
        errors::{Error, JniError, Result as JniResult},
        objects::{GlobalRef, JObject},
        sys::{JNI_VERSION_1_6, jint},
    },
    std::{
        collections::HashMap,
        env::set_var,
        fs::File,
        io::{BufRead, BufReader},
        os::{raw::c_void, unix::prelude::*},
        sync::{LazyLock, Mutex, OnceLock},
        thread::spawn,
    },
};

static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();
static ACTIVITY: Mutex<Option<GlobalRef>> = Mutex::new(None);
static UI_THREAD_PENDINGS: LazyLock<Mutex<HashMap<i64, Box<dyn FnOnce() + Send + Sync>>>> =
    LazyLock::new(Default::default);

#[derive(Clone, Debug)]
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
            if let Ok(mut lock) = ACTIVITY.lock()
                && let Some(vm) = JAVA_VM.get()
                && let Ok(env) = vm.attach_current_thread()
                && let Ok(act) = env.new_global_ref(activity)
            {
                lock.replace(act);
                drop(lock);
                spawn(|| unsafe { main() });
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
extern "C" fn Java_rs_compio_winio_Activity_on_1start(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::Start);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1resume(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::Resume);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1pause(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::Pause);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1stop(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::Stop);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1destroy(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::Destroy);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1configuration_1changed(
    env: JNIEnv,
    activity: JObject,
    new_config: JObject,
) {
    handle_message(
        activity,
        ActivityMessage::ConfigChanged(env.new_global_ref(new_config).unwrap()),
    );
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1low_1memory(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::LowMemory);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1create(_env: JNIEnv, activity: JObject) {
    handle_message(activity, ActivityMessage::Created)
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_1run_1ui_1thread(
    _env: JNIEnv,
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
    F: FnOnce(AttachGuard, GlobalRef) -> JniResult<R>,
{
    if let Some(vm) = JAVA_VM.get() {
        let env = vm.attach_current_thread()?;
        let activity = ACTIVITY
            .lock()
            .map_or(None, |lock| lock.clone())
            .unwrap_or_else(|| env.new_global_ref(JObject::null()).unwrap());
        f(env, activity)
    } else {
        Err(Error::NullPtr("No java VM."))
    }
}

pub fn vm_exec_on_ui_thread<F, R>(f: F) -> JniResult<R>
where
    F: FnOnce(AttachGuard, GlobalRef) -> JniResult<R> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    static mut IDX: i64 = 0;

    vm_exec(|mut env, act| {
        let Ok(mut lock) = UI_THREAD_PENDINGS.lock() else {
            return Err(Error::JniCall(JniError::Unknown));
        };

        let (tx, rx) = oneshot::channel();
        unsafe {
            IDX += 1;
            let act2 = act.clone();
            lock.insert(
                IDX,
                Box::new(move || {
                    let _ = if let Some(vm) = JAVA_VM.get() {
                        let env = match vm.attach_current_thread() {
                            Ok(env) => env,
                            Err(e) => {
                                let _ = tx.send(Err(e));
                                return;
                            }
                        };
                        tx.send(f(env, act2))
                    } else {
                        tx.send(Err(Error::NullPtr("No java VM.")))
                    };
                }),
            );
            drop(lock);
            env.call_method(act.as_obj(), "runOnUiThread", "(J)V", &[IDX.into()])?;
        }
        rx.recv().unwrap()
    })
}
