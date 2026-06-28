use std::{
    cell::RefCell,
    future::Future,
    sync::Mutex,
    task::{RawWaker, RawWakerVTable, Waker},
};

#[doc(no_inline)]
pub use android_activity::AndroidApp;
use android_activity::{MainEvent, PollEvent};
use compio_log::debug;
use jni::{Env, objects::JObject, refs::Global, vm::JavaVM};
use ndk_sys::{ALooper, ALooper_acquire, ALooper_forThread, ALooper_release, ALooper_wake};
use winio_pollable::MainTask;

use crate::{AView, Error, Resources, ResourcesTheme, Result};

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
        self.app.run_on_java_main_thread(Box::new(|| {
            MAIN_TASK.with_borrow(|task| {
                if let Some(task) = task.as_ref() {
                    task.poll();
                }
            });
        }))
    }

    pub fn block_on<F: Future<Output = ()>>(
        &self,
        future: impl (FnOnce() -> F) + Sync + Send + 'static,
    ) {
        let waker = looper_waker();
        self.app.run_on_java_main_thread(Box::new(move || {
            let task = unsafe { MainTask::new(future(), waker) };
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
                        MainEvent::Destroy => {
                            destroyed = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            });
        }
    }
}

static ACTIVITY: Mutex<Option<Global<JObject<'static>>>> = Mutex::new(None);

thread_local! {
    static MAIN_TASK: RefCell<Option<MainTask>> = const { RefCell::new(None) };
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
        ResourcesTheme => "android.content.res.Resources$Theme",
    },
    methods {
        fn get_resources() -> Resources,
        fn get_theme() -> ResourcesTheme,
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
