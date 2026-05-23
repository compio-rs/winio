use std::{
    future::Future,
    os::raw::c_void,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable, Waker},
};

use super::RUNTIME;

const ALOOPER_PREPARE_ALLOW_NON_CALLBACKS: i32 = 1;

#[link(name = "android")]
extern "C" {
    fn ALooper_forThread() -> *mut c_void;
    fn ALooper_prepare(opts: i32) -> *mut c_void;
    fn ALooper_pollOnce(
        timeoutMillis: i32,
        outFd: *mut i32,
        outEvents: *mut i32,
        outData: *mut *mut c_void,
    ) -> i32;
    fn ALooper_wake(looper: *mut c_void);
}

pub struct App {
    looper: NonNull<c_void>,
}

pub type Runtime = App;

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let looper = unsafe {
            let looper = ALooper_forThread();
            if looper.is_null() {
                ALooper_prepare(ALOOPER_PREPARE_ALLOW_NON_CALLBACKS)
            } else {
                looper
            }
        };

        let looper = NonNull::new(looper).expect("failed to prepare Android looper");
        Self { looper }
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        RUNTIME.set(self, f)
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            winio_pollable::block_on(future, looper_waker(self.looper), || unsafe {
                let _ = ALooper_pollOnce(
                    -1,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            })
        })
    }
}

fn looper_waker(looper: NonNull<c_void>) -> Waker {
    unsafe { Waker::from_raw(looper_raw_waker(looper)) }
}

fn looper_raw_waker(looper: NonNull<c_void>) -> RawWaker {
    RawWaker::new(
        looper.as_ptr().cast(),
        &RawWakerVTable::new(
            looper_clone,
            looper_wake,
            looper_wake_by_ref,
            looper_drop,
        ),
    )
}

unsafe fn looper_clone(data: *const ()) -> RawWaker {
    let looper = NonNull::new(data.cast_mut().cast()).expect("looper pointer is null");
    looper_raw_waker(looper)
}

unsafe fn looper_wake(data: *const ()) {
    unsafe { looper_wake_by_ref(data) }
}

unsafe fn looper_wake_by_ref(data: *const ()) {
    if let Some(looper) = NonNull::new(data.cast_mut().cast()) {
        unsafe {
            ALooper_wake(looper.as_ptr());
        }
    }
}

unsafe fn looper_drop(_: *const ()) {}
