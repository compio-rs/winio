use std::{
    future::Future,
    sync::Arc,
    task::{Wake, Waker},
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
        let waker = Waker::from(Arc::new(CFRunLoopWaker::new()?));
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

struct CFRunLoopWaker {
    run_loop: CFRetained<CFRunLoop>,
}

impl CFRunLoopWaker {
    pub fn new() -> Result<Self> {
        let run_loop = CFRunLoop::current().ok_or(Error::NullPointer)?;
        Ok(Self { run_loop })
    }

    fn wake_impl(&self) {
        self.run_loop.wake_up();
    }
}

unsafe impl Send for CFRunLoopWaker {}
unsafe impl Sync for CFRunLoopWaker {}

impl Wake for CFRunLoopWaker {
    fn wake(self: Arc<Self>) {
        self.wake_impl();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_impl();
    }
}
