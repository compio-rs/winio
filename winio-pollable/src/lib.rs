//! A pollable runtime based on [`compio`]. It is just a compio runtime except
//! Linux, where there is also an eventfd if the driver is io-uring. This crate
//! ensures that the raw fd returned by [`RawFd::as_raw_fd`] is always actively
//! pollable.

#![warn(missing_docs)]

#[cfg(target_os = "linux")]
use std::os::fd::OwnedFd;
use std::{
    cell::RefCell,
    future::Future,
    io,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use compio::driver::{AsRawFd, RawFd};
use futures_util::FutureExt;

/// See [`Runtime`]
///
/// [`Runtime`]: compio::runtime::Runtime
pub struct Runtime {
    runtime: compio::runtime::Runtime,
    #[cfg(target_os = "linux")]
    efd: Option<OwnedFd>,
}

#[cfg(target_os = "linux")]
impl Runtime {
    /// Create [`Runtime`].
    pub fn new() -> io::Result<Self> {
        use rustix::event::{EventfdFlags, eventfd};
        let efd = eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?;
        let mut builder = compio::driver::ProactorBuilder::new();
        builder.register_eventfd(efd.as_raw_fd());
        let runtime = compio::runtime::RuntimeBuilder::new()
            .with_proactor(builder)
            .build()?;
        let efd = if runtime.driver_type().is_iouring() {
            Some(efd)
        } else {
            None
        };
        Ok(Self { runtime, efd })
    }

    /// Clear the eventfd, if possible.
    pub(crate) fn clear(&self) -> io::Result<()> {
        if let Some(efd) = &self.efd {
            let mut buf = [0u8; 8];
            rustix::io::read(efd, &mut buf)?;
        }
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
impl Runtime {
    /// Create [`Runtime`].
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            runtime: compio::runtime::Runtime::new()?,
        })
    }

    /// Clear the eventfd, if possible.
    pub(crate) fn clear(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Runtime {
    /// Run the scheduled tasks.
    pub fn run(&self) -> bool {
        let main_task_remaining = if MAIN_TASK.is_set() {
            MAIN_TASK.with(|task| task.poll())
        } else {
            false
        };

        self.runtime.run() | main_task_remaining
    }

    /// Poll the runtime. Returns the next timeout.
    pub fn poll_and_run(&self) -> Option<Duration> {
        self.runtime.poll_with(Some(Duration::ZERO));

        let remaining_tasks = self.run();

        if remaining_tasks {
            Some(Duration::ZERO)
        } else {
            self.runtime.current_timeout()
        }
    }

    /// Set the current main task and wait for its completion.
    pub fn enter_block_on<F: Future<Output = ()>, T>(&self, future: F, f: impl FnOnce() -> T) -> T {
        let waker = self.runtime.waker();
        let task = unsafe { MainTask::new(future, waker) };
        MAIN_TASK.set(&task, f)
    }

    /// Block on the future till it completes. Users should enter the runtime
    /// before calling this function, and poll the runtime themselves.
    pub fn block_on<F: Future>(&self, future: F, poll: impl Fn(Option<Duration>)) -> F::Output {
        let result = RefCell::new(None);

        self.enter_block_on(
            async {
                let res = Some(future.await);
                result.replace(res);
            },
            || {
                loop {
                    let timeout = self.poll_and_run();

                    if let Some(result) = result.take() {
                        break result;
                    }

                    poll(timeout);

                    self.clear().ok();
                }
            },
        )
    }
}

impl Deref for Runtime {
    type Target = compio::runtime::Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl DerefMut for Runtime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime
    }
}

impl AsRawFd for Runtime {
    fn as_raw_fd(&self) -> RawFd {
        #[cfg(target_os = "linux")]
        {
            self.efd
                .as_ref()
                .map(|f| f.as_raw_fd())
                .unwrap_or_else(|| self.runtime.as_raw_fd())
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.runtime.as_raw_fd()
        }
    }
}

struct MainTask {
    future: RefCell<Pin<Box<dyn Future<Output = ()>>>>,
    waker: Waker,
}

unsafe fn reduce_lifetime<'a>(
    future: impl Future<Output = ()> + 'a,
) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
    let future = Box::pin(future) as Pin<Box<dyn Future<Output = ()> + 'a>>;
    unsafe {
        std::mem::transmute::<
            Pin<Box<dyn Future<Output = ()> + 'a>>,
            Pin<Box<dyn Future<Output = ()> + 'static>>,
        >(future)
    }
}

impl MainTask {
    pub unsafe fn new(future: impl Future<Output = ()>, waker: Waker) -> Self {
        Self {
            // SAFETY: the future will only be polled within the scope of `enter_block_on`, which
            // guarantees that the future will not outlive the main task.
            future: RefCell::new(unsafe { reduce_lifetime(future.fuse()) }),
            waker,
        }
    }

    pub fn poll(&self) -> bool {
        let mut cx = Context::from_waker(&self.waker);
        if let Ok(mut fut) = self.future.try_borrow_mut() {
            if let Poll::Ready(()) = fut.as_mut().poll(&mut cx) {
                // The future has completed, so we should not wait for the driver to wake us up
                // anymore.
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

scoped_tls::scoped_thread_local!(static MAIN_TASK: MainTask);
