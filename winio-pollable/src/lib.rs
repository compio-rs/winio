//! A pollable runtime based on [`compio`]. It is just a compio runtime except
//! Linux, where there is also an eventfd if the driver is io-uring. This crate
//! ensures that the raw fd returned by [`RawFd::as_raw_fd`] is always actively
//! pollable.

#![warn(missing_docs)]

#[cfg(target_os = "linux")]
use std::os::fd::OwnedFd;
use std::{
    io,
    ops::{Deref, DerefMut},
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
        let efd = if compio::driver::DriverType::is_iouring() {
            use rustix::event::{EventfdFlags, eventfd};
            Some(eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?)
        } else {
            None
        };
        let mut builder = compio::driver::ProactorBuilder::new();
        if let Some(fd) = &efd {
            builder.register_eventfd(fd.as_raw_fd());
        }
        let runtime = compio::runtime::RuntimeBuilder::new()
            .with_proactor(builder)
            .build()?;
        Ok(Self { runtime, efd })
    }

    /// Clear the eventfd, if possible.
    pub fn clear(&self) -> io::Result<()> {
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
    pub fn clear(&self) -> io::Result<()> {
        Ok(())
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
