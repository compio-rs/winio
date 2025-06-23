use std::{future::Future, os::fd::OwnedFd, time::Duration};

use compio::driver::{AsRawFd, DriverType};
use gtk4::{
    gio::{self, prelude::ApplicationExt},
    glib::{ControlFlow, IOCondition, MainContext, timeout_add_local_once, unix_fd_add_local},
};

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    app: gio::Application,
    #[cfg(target_os = "linux")]
    efd: Option<OwnedFd>,
    ctx: MainContext,
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = compio::runtime::Runtime::new().unwrap();
        #[cfg(target_os = "linux")]
        let (efd, poll_fd) = {
            let efd = if DriverType::current() == DriverType::IoUring {
                Some(super::iour::register_eventfd(runtime.as_raw_fd()).unwrap())
            } else {
                None
            };
            let poll_fd = efd
                .as_ref()
                .map(|f| f.as_raw_fd())
                .unwrap_or_else(|| runtime.as_raw_fd());
            (efd, poll_fd)
        };
        #[cfg(not(target_os = "linux"))]
        let poll_fd = runtime.as_raw_fd();
        let ctx = MainContext::default();
        gtk4::init().unwrap();
        let app = gio::Application::new(None, gio::ApplicationFlags::FLAGS_NONE);
        app.set_default();

        unix_fd_add_local(poll_fd, IOCondition::IN, |_fd, _cond| ControlFlow::Continue);

        Self {
            runtime,
            app,
            #[cfg(target_os = "linux")]
            efd,
            ctx,
        }
    }

    pub fn set_app_id(&mut self, name: &str) {
        self.app.set_application_id(Some(name));
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
                let source_id = timeout.map(|timeout| timeout_add_local_once(timeout, || {}));

                self.ctx.iteration(true);

                if let Some(source_id) = source_id {
                    if self.ctx.find_source_by_id(&source_id).is_some() {
                        source_id.remove();
                    }
                }

                #[cfg(target_os = "linux")]
                if let Some(efd) = &self.efd {
                    super::iour::eventfd_clear(efd.as_raw_fd()).ok();
                }
            }
        })
    }
}
