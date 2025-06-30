use std::{cell::RefCell, future::Future, os::fd::AsRawFd, time::Duration};

use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(args: Vec<String>, fd: i32) -> UniquePtr<WinioQtEventLoop>;

        fn process(self: Pin<&mut Self>);
        #[rust_name = "process_timeout"]
        fn process(self: Pin<&mut Self>, maxtime: i32);

        fn setAppName(self: Pin<&mut Self>, name: &str);
    }
}

pub struct Runtime {
    runtime: winio_pollable::Runtime,
    event_loop: RefCell<UniquePtr<ffi::WinioQtEventLoop>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = winio_pollable::Runtime::new().unwrap();
        let poll_fd = runtime.as_raw_fd();
        let args = std::env::args().collect::<Vec<_>>();
        let event_loop = RefCell::new(ffi::new_event_loop(args, poll_fd));

        Self {
            runtime,
            event_loop,
        }
    }

    pub fn set_app_id(&mut self, id: &str) {
        self.event_loop.borrow_mut().pin_mut().setAppName(id);
    }

    pub(crate) fn run(&self) {
        self.runtime.run();
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| crate::RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let mut result = None;
            unsafe {
                self.runtime
                    .spawn_unchecked(async { result = Some(future.await) })
            }
            .detach();
            loop {
                self.runtime.poll_with(Some(Duration::ZERO));

                let remaining_tasks = self.runtime.run();
                if let Some(result) = result.take() {
                    break result;
                }

                let timeout = if remaining_tasks {
                    Some(Duration::ZERO)
                } else {
                    self.runtime.current_timeout()
                };

                if let Some(timeout) = timeout {
                    self.event_loop
                        .borrow_mut()
                        .pin_mut()
                        .process_timeout(timeout.as_millis() as _);
                } else {
                    self.event_loop.borrow_mut().pin_mut().process();
                }

                self.runtime.clear().ok();
            }
        })
    }
}
