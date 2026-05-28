use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{RawWaker, RawWakerVTable, Waker},
};

use android_activity::{AndroidApp, MainEvent, PollEvent};
use futures_util::FutureExt;
use jni::{Env, objects::JObject, refs::Global};
use jni_min_helper::jni_get_vm;
use ndk_sys::{ALooper, ALooper_acquire, ALooper_forThread, ALooper_release, ALooper_wake};
use slab::Slab;
use winio_callback::{Callback, Runnable};

use crate::{Error, GlobalRef, Result};

pub struct App {
    app: AndroidApp,
}

impl App {
    pub fn new(app: AndroidApp) -> Result<Self> {
        Ok(Self { app })
    }

    pub(crate) fn activity(&self, env: &Env<'_>) -> Result<GlobalRef> {
        let activity = self.app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&activity)? };
        Ok(env.new_global_ref(&activity)?)
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        APP.set(self, || {
            let result = RefCell::new(None);
            winio_pollable::enter_block_on(
                future.map(|res| {
                    result.replace(Some(res));
                }),
                looper_waker(),
                || {
                    let mut destroyed = false;
                    while !destroyed {
                        self.app.poll_events(None, |event| {
                            compio_log::debug!("Event: {:?}", event);
                            match event {
                                PollEvent::Wake | PollEvent::Timeout => {
                                    winio_pollable::run_current_task();
                                }
                                PollEvent::Main(e) => match e {
                                    MainEvent::Start => {
                                        winio_pollable::run_current_task();
                                    }
                                    MainEvent::ConfigChanged { .. } if signal_resize::<()>() => {
                                        winio_pollable::run_current_task();
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
                    result.take().expect("app exited without returning a value")
                },
            )
        })
    }
}

scoped_tls::scoped_thread_local!(pub(crate) static APP: App);

thread_local! {
    pub(crate) static RESIZE_SLAB: RefCell<Slab<Rc<Callback>>> = const { RefCell::new(Slab::new()) };
}

fn signal_resize<R: Runnable>() -> bool {
    RESIZE_SLAB.with_borrow(|s| {
        for (_, callback) in s.iter() {
            callback.signal::<R>(());
        }
        !s.is_empty()
    })
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

pub fn vm_exec<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Env<'_>) -> Result<R>,
{
    let vm = jni_get_vm();
    vm.attach_current_thread::<_, R, Error>(f)
}

pub fn current_activity() -> Result<GlobalRef> {
    vm_exec(|env| APP.with(|app| app.activity(env)))
}
