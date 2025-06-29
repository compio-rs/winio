use std::{future::Future, os::fd::OwnedFd, time::Duration};

use compio::driver::AsRawFd;
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

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        #[cfg(not(target_os = "linux"))]
        let (runtime, efd, poll_fd) = {
            let runtime = compio::runtime::Runtime::new().unwrap();
            (runtime, None, runtime.as_raw_fd())
        };

        #[cfg(target_os = "linux")]
        let (runtime, efd, poll_fd) = {
            let efd = if compio::driver::DriverType::is_iouring() {
                use rustix::event::{EventfdFlags, eventfd};
                eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK).ok()
            } else {
                None
            };
            let mut builder = compio::driver::ProactorBuilder::new();
            if let Some(fd) = &efd {
                builder.register_eventfd(fd.as_raw_fd());
            }
            let runtime = compio::runtime::RuntimeBuilder::new()
                .with_proactor(builder)
                .build()
                .unwrap();
            let poll_fd = efd
                .as_ref()
                .map(|f| f.as_raw_fd())
                .unwrap_or_else(|| runtime.as_raw_fd());
            (runtime, efd, poll_fd)
        };
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

    pub(crate) fn run(&self) {
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
                    let mut buf = [0u8; 4];
                    rustix::io::read(efd, &mut buf).ok();
                }
            }
        })
    }
}
