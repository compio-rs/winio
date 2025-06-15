use std::{cell::RefCell, future::Future, os::fd::OwnedFd, time::Duration};

use compio::driver::{AsRawFd, DriverType};
use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(args: Vec<String>, fd: i32) -> UniquePtr<WinioQtEventLoop>;

        fn process(self: Pin<&mut Self>);
        #[rust_name = "process_timeout"]
        fn process(self: Pin<&mut Self>, maxtime: i32);

        fn setAppName(self: Pin<&mut Self>, name: &str);
    }
}

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    efd: Option<OwnedFd>,
    event_loop: RefCell<UniquePtr<ffi::WinioQtEventLoop>>,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();
        let efd = if DriverType::current() == DriverType::IoUring {
            Some(super::iour::register_eventfd(runtime.as_raw_fd()).unwrap())
        } else {
            None
        };
        let poll_fd = efd
            .as_ref()
            .map(|f| f.as_raw_fd())
            .unwrap_or_else(|| runtime.as_raw_fd());
        let args = std::env::args().collect::<Vec<_>>();
        let event_loop = RefCell::new(ffi::new_event_loop(args, poll_fd));

        Self {
            runtime,
            efd,
            event_loop,
        }
    }

    pub fn set_app_id(&mut self, id: &str) {
        self.event_loop.borrow_mut().pin_mut().setAppName(id);
    }

    pub fn run(&self) {
        self.runtime.run();
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| super::RUNTIME.set(self, f))
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

                if let Some(efd) = &self.efd {
                    super::iour::eventfd_clear(efd.as_raw_fd()).ok();
                }
            }
        })
    }
}
