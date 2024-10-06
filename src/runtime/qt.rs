use std::{cell::RefCell, future::Future, time::Duration};

use compio::driver::AsRawFd;
use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(args: Vec<String>, fd: i32) -> UniquePtr<WinioQtEventLoop>;

        #[allow(dead_code)]
        fn process(self: Pin<&mut Self>);
        #[rust_name = "process_timeout"]
        fn process(self: Pin<&mut Self>, maxtime: i32);
    }
}

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    event_loop: RefCell<UniquePtr<ffi::WinioQtEventLoop>>,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();
        let args = std::env::args().collect::<Vec<_>>();
        let event_loop = RefCell::new(ffi::new_event_loop(args, runtime.as_raw_fd()));

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

                // TODO: remove this workaround
                let max_timeout = Duration::from_millis(100);
                let timeout = timeout
                    .map(|timeout| timeout.min(max_timeout))
                    .unwrap_or(max_timeout);

                self.event_loop
                    .borrow_mut()
                    .pin_mut()
                    .process_timeout(timeout.as_millis() as _);
            }
        })
    }
}
