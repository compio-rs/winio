use std::{future::Future, os::raw::c_void, ptr::null, time::Duration};

use compio::driver::AsRawFd;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSEventMask};
use objc2_core_foundation::{
    CFFileDescriptor, CFRetained, CFRunLoop, kCFAllocatorDefault, kCFFileDescriptorReadCallBack,
    kCFRunLoopDefaultMode,
};
use objc2_foundation::{MainThreadMarker, NSDate, NSDefaultRunLoopMode};

pub struct Runtime {
    runtime: winio_pollable::Runtime,
    fd_source: CFRetained<CFFileDescriptor>,
    ns_app: Retained<NSApplication>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = winio_pollable::Runtime::new().unwrap();

        unsafe extern "C-unwind" fn callback(
            _fdref: *mut CFFileDescriptor,
            _callback_types: usize,
            _info: *mut c_void,
        ) {
        }

        let fd_source = unsafe {
            CFFileDescriptor::new(
                kCFAllocatorDefault,
                runtime.as_raw_fd(),
                false,
                Some(callback),
                null(),
            )
        }
        .unwrap();
        let source = unsafe {
            CFFileDescriptor::new_run_loop_source(kCFAllocatorDefault, Some(&fd_source), 0)
        }
        .unwrap();

        unsafe {
            let run_loop = CFRunLoop::current().unwrap();
            run_loop.add_source(Some(&source), kCFRunLoopDefaultMode);
        }

        let ns_app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
        ns_app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        #[allow(deprecated)]
        ns_app.activateIgnoringOtherApps(true);
        Self {
            runtime,
            fd_source,
            ns_app,
        }
    }

    pub(crate) fn run(&self) {
        self.runtime.run();
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| super::RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            self.runtime.block_on(future, |timeout| {
                self.fd_source
                    .enable_call_backs(kCFFileDescriptorReadCallBack);
                CFRunLoop::run_in_mode(
                    unsafe { kCFRunLoopDefaultMode },
                    timeout.unwrap_or(Duration::MAX).as_secs_f64(),
                    true,
                );
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
        })
    }
}
