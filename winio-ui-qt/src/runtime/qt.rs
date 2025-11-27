use std::{cell::RefCell, future::Future, os::fd::AsRawFd};

use cxx::UniquePtr;

use crate::Result;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(args: Vec<String>, fd: i32) -> Result<UniquePtr<WinioQtEventLoop>>;

        fn process(self: Pin<&mut Self>);
        #[rust_name = "process_timeout"]
        fn process(self: Pin<&mut Self>, maxtime: i32);

        fn setAppName(self: Pin<&mut Self>, name: &str) -> Result<()>;
    }
}

pub struct Runtime {
    runtime: winio_pollable::Runtime,
    event_loop: RefCell<UniquePtr<ffi::WinioQtEventLoop>>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let runtime = winio_pollable::Runtime::new()?;
        let poll_fd = runtime.as_raw_fd();
        let args = std::env::args().collect::<Vec<_>>();
        let event_loop = RefCell::new(ffi::new_event_loop(args, poll_fd)?);

        Ok(Self {
            runtime,
            event_loop,
        })
    }

    pub fn set_app_id(&mut self, id: &str) -> Result<()> {
        self.event_loop.borrow_mut().pin_mut().setAppName(id)?;
        Ok(())
    }

    pub(crate) fn run(&self) {
        self.runtime.run();
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| crate::RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            self.runtime.block_on(future, |timeout| {
                if let Some(timeout) = timeout {
                    self.event_loop
                        .borrow_mut()
                        .pin_mut()
                        .process_timeout(timeout.as_millis() as _);
                } else {
                    self.event_loop.borrow_mut().pin_mut().process();
                }
            })
        })
    }
}
