use std::{cell::Cell, future::Future, time::Duration};

use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn poll_runtime() -> bool;
    }
    unsafe extern "C++" {
        include!("winio/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(args: Vec<String>) -> UniquePtr<WinioQtEventLoop>;

        fn exec();
    }
}

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    #[allow(dead_code)]
    event_loop: UniquePtr<ffi::WinioQtEventLoop>,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();
        let args = std::env::args().collect::<Vec<_>>();
        let event_loop = ffi::new_event_loop(args);

        Self {
            runtime,
            event_loop,
        }
    }

    pub fn run(&self) {
        self.runtime.run();
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| super::RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let completed = Cell::new(false);
            BLOCK_ON_COMPLETED.set(&completed, || {
                let mut result = None;
                unsafe {
                    self.runtime.spawn_unchecked(async {
                        result = Some(future.await);
                        completed.set(true);
                    })
                }
                .detach();
                loop {
                    ffi::exec();
                    self.runtime.run();
                    if let Some(result) = result.take() {
                        break result;
                    }
                }
            })
        })
    }
}

scoped_tls::scoped_thread_local!(static BLOCK_ON_COMPLETED: Cell<bool>);

fn poll_runtime() -> bool {
    const MAX_TIMEOUT: Duration = Duration::from_millis(100);

    super::RUNTIME.with(|runtime| {
        let remaining = runtime.runtime.run();
        let timeout = if remaining {
            Duration::ZERO
        } else {
            // wait for a short time
            runtime
                .runtime
                .current_timeout()
                .map(|t| t.min(MAX_TIMEOUT))
                .unwrap_or(MAX_TIMEOUT)
        };
        runtime.runtime.poll_with(Some(timeout));
        BLOCK_ON_COMPLETED.with(|b| b.get())
    })
}
