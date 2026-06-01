use std::{
    cell::RefCell,
    future::Future,
    sync::{Arc, Mutex},
    task::{RawWaker, RawWakerVTable, Waker},
};

use android_activity::{AndroidApp, MainEvent, PollEvent};
use compio_log::{debug, error, warn};
use futures_util::FutureExt;
use jni::{Env, objects::JObject, refs::Global, vm::JavaVM};
use jni_min_helper::DynamicProxy;
use ndk_sys::{ALooper, ALooper_acquire, ALooper_forThread, ALooper_release, ALooper_wake};
use oneshot::TryRecvError;
use winio_callback::SyncCallback;
use winio_pollable::MainTask;

use crate::{AView, Error, Resources, Result};

pub struct App {
    app: AndroidApp,
}

impl App {
    pub fn new(app: AndroidApp) -> Result<Self> {
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
        let activity = vm.attach_current_thread(|env| Self::activity(&app, env))?;
        ACTIVITY.lock().unwrap().replace(activity);
        Ok(Self { app })
    }

    pub(crate) fn activity(app: &AndroidApp, env: &Env<'_>) -> Result<Global<JObject<'static>>> {
        let activity = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&activity)? };
        Ok(env.new_global_ref(&activity)?)
    }

    fn run_current_task(&self) {
        let res = DynamicProxy::post_to_main_looper(|_env| {
            MAIN_TASK.with_borrow(|task| {
                if let Some(task) = task.as_ref() {
                    task.poll();
                }
            });
            Ok(())
        });
        match res {
            Ok(true) => {}
            Ok(false) => {
                warn!("cannot run task on main looper");
            }
            Err(e) => {
                error!("Error posting task to main looper: {e:?}");
            }
        }
    }

    pub fn block_on<F: Future<Output = ()>>(
        &self,
        future: impl (FnOnce() -> F) + Sync + Send + 'static,
    ) {
        let (tx, rx) = oneshot::channel();

        let waker = looper_waker();
        self.app.run_on_java_main_thread(Box::new(move || {
            let future = future().map(move |res| {
                tx.send(res).ok();
            });
            let task = unsafe { MainTask::new(future, waker) };
            MAIN_TASK.with_borrow_mut(|t| t.replace(task));
        }));

        let mut destroyed = false;
        while !destroyed {
            self.app.poll_events(None, |event| {
                debug!("Event: {:?}", event);
                match event {
                    PollEvent::Wake | PollEvent::Timeout => {
                        self.run_current_task();
                    }
                    PollEvent::Main(e) => match e {
                        MainEvent::Start
                        | MainEvent::Resume { .. }
                        | MainEvent::ConfigChanged { .. } => {
                            self.run_current_task();
                        }
                        MainEvent::Stop if signal_destroy() => {
                            self.run_current_task();
                        }
                        MainEvent::Destroy => {
                            destroyed = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            });
        }
        signal_destroy();
        let mut counter = 0;
        loop {
            match rx.try_recv() {
                Ok(result) => break result,
                Err(TryRecvError::Empty) => {
                    self.run_current_task();
                    counter += 1;
                    if counter > 64 {
                        panic!("Main task is taking a long time to complete");
                    }
                    std::thread::yield_now();
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Main task was dropped without completing");
                }
            }
        }
    }
}

static ACTIVITY: Mutex<Option<Global<JObject<'static>>>> = Mutex::new(None);

thread_local! {
    static MAIN_TASK: RefCell<Option<MainTask>> = const { RefCell::new(None) };
}

pub(crate) static DESTROY_CALLBACK: Mutex<Option<Arc<SyncCallback>>> = Mutex::new(None);

fn signal_destroy() -> bool {
    let s = DESTROY_CALLBACK.lock().unwrap();
    if let Some(callback) = s.as_ref() {
        callback.signal(());
        true
    } else {
        false
    }
}

fn looper_waker() -> Waker {
    let looper = unsafe { ALooper_forThread() };
    unsafe { ALooper_acquire(looper) };
    unsafe { Waker::from_raw(looper_raw_waker(looper)) }
}

fn looper_raw_waker(looper: *mut ALooper) -> RawWaker {
    RawWaker::new(
        looper.cast(),
        &RawWakerVTable::new(looper_clone, looper_wake, looper_wake_by_ref, looper_drop),
    )
}

unsafe fn looper_clone(data: *const ()) -> RawWaker {
    let looper = data.cast_mut().cast();
    unsafe { ALooper_acquire(looper) };
    looper_raw_waker(looper)
}

unsafe fn looper_wake(data: *const ()) {
    unsafe {
        looper_wake_by_ref(data);
        looper_drop(data);
    }
}

unsafe fn looper_wake_by_ref(data: *const ()) {
    let looper = data.cast_mut().cast();
    unsafe { ALooper_wake(looper) }
}

unsafe fn looper_drop(data: *const ()) {
    let looper = data.cast_mut().cast();
    unsafe { ALooper_release(looper) }
}

pub(crate) fn vm_exec<F, R, E>(f: F) -> std::result::Result<R, E>
where
    F: FnOnce(&mut Env<'_>) -> std::result::Result<R, E>,
    E: From<jni::errors::Error>,
{
    let vm = JavaVM::singleton()?;
    vm.attach_current_thread::<_, R, E>(f)
}

jni::bind_java_type! {
    pub(crate) Context => android.content.Context,
    type_map {
        Resources => android.content.res.Resources,
    },
    methods {
        fn get_resources() -> Resources,
    }
}

jni::bind_java_type! {
    pub(crate) Activity => rs.compio.winio.Activity,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
    },
    methods {
        fn set_content_view(view: &AView),
    },
    native_methods {
        extern fn on_create_native(),
    },
    is_instance_of = {
        context = Context,
    }
}

pub(crate) fn current_activity<'local>(env: &mut Env<'local>) -> Result<Activity<'local>> {
    let act = ACTIVITY.lock().unwrap();
    let obj = env.new_local_ref(act.as_ref().ok_or(Error::NoApp)?.as_obj())?;
    Ok(unsafe { Activity::from_raw(env, obj.into_raw()) })
}

impl ActivityNativeInterface for ActivityAPI {
    type Error = jni::errors::Error;

    fn on_create_native<'local>(
        env: &mut Env<'local>,
        this: Activity<'local>,
    ) -> std::result::Result<(), Self::Error> {
        crate::register_launcher(env, &this)
    }
}
