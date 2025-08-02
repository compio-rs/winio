use {
    super::RUNTIME,
    std::{future::Future, thread::sleep, time::Duration},
    winio_pollable::Runtime as PollableRuntime,
};

pub struct Runtime {
    inner: PollableRuntime,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let inner = PollableRuntime::new().unwrap();

        Self { inner }
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.inner.enter(|| RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let mut result = None;
            unsafe {
                self.runtime.spawn_unchecked(async {
                    result = Some(future.await);
                    // exit the application here, if possible
                })
            }
            .detach();
            // start the application here and wait till it exits
            result.unwrap()
        })
    }
}
