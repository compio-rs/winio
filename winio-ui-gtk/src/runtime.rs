use std::{future::Future, os::fd::AsRawFd};

use gtk4::{
    gio::{self, prelude::ApplicationExt},
    glib::{ControlFlow, IOCondition, MainContext, timeout_add_local_once, unix_fd_add_local},
};

pub struct Runtime {
    runtime: winio_pollable::Runtime,
    app: gio::Application,
    ctx: MainContext,
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
        let ctx = MainContext::default();
        gtk4::init().unwrap();
        let app = gio::Application::new(None, gio::ApplicationFlags::FLAGS_NONE);
        app.set_default();

        unix_fd_add_local(poll_fd, IOCondition::IN, |_fd, _cond| ControlFlow::Continue);

        Self { runtime, app, ctx }
    }

    pub fn set_app_id(&mut self, name: &str) {
        self.app.set_application_id(Some(name));
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
                let source_id = timeout.map(|timeout| timeout_add_local_once(timeout, || {}));

                self.ctx.iteration(true);

                if let Some(source_id) = source_id {
                    if self.ctx.find_source_by_id(&source_id).is_some() {
                        source_id.remove();
                    }
                }
            })
        })
    }
}
