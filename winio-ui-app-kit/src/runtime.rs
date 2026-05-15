use std::{
    future::Future,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable, Waker},
    time::Duration,
};

use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSEventMask};
use objc2_core_foundation::{CFRetained, CFRunLoop, kCFRunLoopDefaultMode};
use objc2_foundation::{MainThreadMarker, NSDate, NSDefaultRunLoopMode};

#[cfg(feature = "compio-compat")]
use crate::get_context;
use crate::{Error, Result, catch};

#[cfg(not(feature = "compio-compat"))]
fn get_context() -> (Option<Duration>, Option<Waker>) {
    (None, None)
}

pub struct App {
    ns_app: Retained<NSApplication>,
    waker: Waker,
}

impl App {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        let ns_app = catch(|| {
            let ns_app = NSApplication::sharedApplication(mtm);
            ns_app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
            #[allow(deprecated)]
            ns_app.activateIgnoringOtherApps(true);
            ns_app
        })?;
        let waker = run_loop_waker(CFRunLoop::current().ok_or(Error::NullPointer)?);
        Ok(Self { ns_app, waker })
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        winio_pollable::block_on(future, self.waker.clone(), || {
            let (timeout, waker) = get_context();
            CFRunLoop::run_in_mode(
                unsafe { kCFRunLoopDefaultMode },
                timeout.unwrap_or(Duration::MAX).as_secs_f64(),
                true,
            );
            if let Some(waker) = waker {
                waker.wake();
            }
            unsafe {
                loop {
                    let event = self.ns_app.nextEventMatchingMask_untilDate_inMode_dequeue(
                        NSEventMask::Any,
                        Some(&NSDate::distantPast()),
                        NSDefaultRunLoopMode,
                        true,
                    );
                    if let Some(event) = event {
                        self.ns_app.sendEvent(&event);
                    } else {
                        break;
                    }
                }
            }
        })
    }
}

fn run_loop_waker(run_loop: CFRetained<CFRunLoop>) -> Waker {
    unsafe { Waker::from_raw(run_loop_raw_waker(run_loop)) }
}

fn run_loop_raw_waker(run_loop: CFRetained<CFRunLoop>) -> RawWaker {
    let data = CFRetained::into_raw(run_loop);
    RawWaker::new(
        data.as_ptr().cast_const().cast(),
        &RawWakerVTable::new(
            run_loop_clone,
            run_loop_wake,
            run_loop_wake_by_ref,
            run_loop_drop,
        ),
    )
}

unsafe fn run_loop_clone(data: *const ()) -> RawWaker {
    let data = NonNull::new(data.cast_mut().cast()).expect("data pointer is null");
    let run_loop = unsafe { CFRetained::<CFRunLoop>::retain(data) };
    run_loop_raw_waker(run_loop)
}

unsafe fn run_loop_wake(data: *const ()) {
    if let Some(data) = NonNull::new(data.cast_mut().cast()) {
        let run_loop = unsafe { CFRetained::<CFRunLoop>::from_raw(data) };
        run_loop.wake_up();
    }
}

unsafe fn run_loop_wake_by_ref(data: *const ()) {
    if let Some(data) = NonNull::new(data.cast_mut().cast()) {
        let run_loop = unsafe { CFRetained::<CFRunLoop>::retain(data) };
        run_loop.wake_up();
    }
}

unsafe fn run_loop_drop(data: *const ()) {
    if let Some(data) = NonNull::new(data.cast_mut().cast()) {
        let _ = unsafe { CFRetained::<CFRunLoop>::from_raw(data) };
    }
}
