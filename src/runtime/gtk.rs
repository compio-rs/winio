use std::{future::Future, time::Duration};

use compio::driver::AsRawFd;
use gtk4::glib::{
    timeout_add_local_once, unix_fd_add_local, ControlFlow, IOCondition, MainContext,
};

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    ctx: MainContext,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();
        let ctx = MainContext::default();
        gtk4::init().unwrap();

        unix_fd_add_local(runtime.as_raw_fd(), IOCondition::IN, |_fd, _cond| {
            ControlFlow::Continue
        });

        Self { runtime, ctx }
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

            let timeout = self.runtime.current_timeout();
            let source_id = timeout.map(|timeout| timeout_add_local_once(timeout, || {}));

            self.ctx.iteration(true);

            if let Some(source_id) = source_id {
                if self.ctx.find_source_by_id(&source_id).is_some() {
                    source_id.remove();
                }
            }
        }
    }
}
