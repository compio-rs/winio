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

use crate::{
    Error, Result,
    java::custom::{Activity, ActivityAPI, ActivityNativeInterface},
};

pub struct App {
    app: AndroidApp,
}

impl App {
    pub fn new(app: AndroidApp) -> Result<Self> {
        let _vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
        APP.lock().unwrap().replace(app.clone());
        Ok(Self { app })
    }

    fn run_current_task(&self) {
        wake_main_task();
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

static APP: Mutex<Option<AndroidApp>> = Mutex::new(None);

thread_local! {
    static MAIN_TASK: RefCell<Option<MainTask>> = const { RefCell::new(None) };
}

fn poll_main_task() {
    MAIN_TASK.with_borrow(|task| {
        if let Some(task) = task.as_ref() {
            task.poll();
        }
    });
}

fn wake_main_task() {
    let app = APP.lock().unwrap().clone();
    if let Some(app) = app {
        app.run_on_java_main_thread(Box::new(poll_main_task));
    }
}

pub(crate) struct AndroidRuntime;

impl winio_callback::Runnable for AndroidRuntime {
    fn run() {
        poll_main_task();
    }
}

impl winio_callback::SyncRunnable for AndroidRuntime {
    fn run() {
        wake_main_task();
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

pub(crate) fn current_activity<'local>(env: &mut Env<'local>) -> Result<Activity<'local>> {
    let app = APP.lock().unwrap();
    let app = app.as_ref().ok_or(Error::NoApp)?;
    let activity = app.activity_as_ptr() as jni::sys::jobject;
    let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&activity)? };
    let obj = env.new_local_ref(activity)?;
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
