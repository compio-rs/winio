use std::future::Future;

use crate::{sys, sys::Runtime};

/// Root application, manages the async runtime.
pub struct App {
    runtime: Runtime,
    name: String,
}

impl App {
    /// Create [`App`] with application name.
    pub fn new(name: impl AsRef<str>) -> sys::Result<Self> {
        #[allow(unused_mut)]
        let mut runtime = Runtime::new()?;
        let name = name.as_ref().to_string();
        #[cfg(not(any(windows, target_vendor = "apple")))]
        runtime.set_app_id(&name)?;
        Ok(Self { runtime, name })
    }

    /// The application name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Block on the future till it completes.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}
