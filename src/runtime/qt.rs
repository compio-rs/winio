use std::{future::Future, time::Duration};

use compio::driver::AsRawFd;
use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(fd: i32) -> UniquePtr<WinioQtEventLoop>;

        fn process(&self);
        #[rust_name = "process_timeout"]
        fn process(&self, maxtime: i32);
    }
}

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    event_loop: UniquePtr<ffi::WinioQtEventLoop>,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();
        let event_loop = ffi::new_event_loop(runtime.as_raw_fd());

        Self {
            runtime,
            event_loop,
        }
    }

    pub fn run(&self) {
        self.runtime.run();
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.enter(|| {
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
                    self.event_loop.process_timeout(timeout.as_millis() as _);
                } else {
                    self.event_loop.process();
                }
            }
        })
    }
}
