use std::{future::Future, os::raw::c_void, time::Duration};

use compio::driver::AsRawFd;
use core_foundation::{
    filedescriptor::{kCFFileDescriptorReadCallBack, CFFileDescriptor, CFFileDescriptorRef},
    runloop::{kCFRunLoopDefaultMode, CFRunLoop},
};
use icrate::{
    objc2::rc::Id,
    AppKit::{NSApplication, NSApplicationActivationPolicyRegular, NSEventMaskAny},
    Foundation::{MainThreadMarker, NSDate, NSDefaultRunLoopMode},
};

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    fd_source: CFFileDescriptor,
    ns_app: Id<NSApplication>,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();

        extern "C" fn callback(
            _fdref: CFFileDescriptorRef,
            _callback_types: usize,
            _info: *mut c_void,
        ) {
        }

        let fd_source = CFFileDescriptor::new(runtime.as_raw_fd(), false, callback, None).unwrap();
        let source = fd_source.to_run_loop_source(0).unwrap();

        CFRunLoop::get_current().add_source(&source, unsafe { kCFRunLoopDefaultMode });

        let ns_app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
        ns_app.setActivationPolicy(NSApplicationActivationPolicyRegular);
        #[allow(deprecated)]
        ns_app.activateIgnoringOtherApps(true);
        Self {
            runtime,
            fd_source,
            ns_app,
        }
    }

    pub fn run(&self) {
        self.runtime.run();
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let _guard = self.runtime.enter();
        let mut result = None;
        unsafe {
            self.runtime
                .spawn_unchecked(async { result = Some(future.await) })
        }
        .detach();
        loop {
            self.runtime.run();
            if let Some(result) = result.take() {
                break result;
            }

            self.runtime.poll_with(Some(Duration::ZERO));
            self.fd_source
                .enable_callbacks(kCFFileDescriptorReadCallBack);
            CFRunLoop::run_in_mode(
                unsafe { kCFRunLoopDefaultMode },
                self.runtime.current_timeout().unwrap_or(Duration::MAX),
                true,
            );
            unsafe {
                loop {
                    let event = self.ns_app.nextEventMatchingMask_untilDate_inMode_dequeue(
                        NSEventMaskAny,
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
        }
    }
}
