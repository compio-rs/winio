//! A pollable runtime based on [`compio`]. It is just a compio runtime except
//! Linux, where there is also an eventfd if the driver is io-uring. This crate
//! ensures that the raw fd returned by [`RawFd::as_raw_fd`] is always actively
//! pollable.

#![warn(missing_docs)]

#[cfg(target_os = "linux")]
use std::os::fd::OwnedFd;
use std::{
    future::Future,
    io,
    ops::{Deref, DerefMut},
    time::Duration,
};

use compio::driver::{AsRawFd, RawFd};

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
    /// Block on the future till it completes. Users should enter the runtime
    /// before calling this function, and poll the runtime themselves.
    pub fn block_on<F: Future>(&self, future: F, poll: impl Fn(Option<Duration>)) -> F::Output {
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

            poll(timeout);

            self.clear().ok();
        }
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
