use std::future::Future;

use compio_log::{error, warn};
use futures_util::StreamExt;
use winio_elm::{Component, Root, RunEvent};
#[cfg(feature = "compio-compat")]
use {
    compio::{compat::RuntimeCompat, runtime::Runtime},
    sys::CompioAdapter,
};

use crate::{sys, sys::App as SysApp};

/// Root application, manages the async runtime.
pub struct App {
    #[cfg(feature = "compio-compat")]
    runtime: RuntimeCompat<CompioAdapter>,
    app: SysApp,
    name: String,
}

impl App {
    /// Create [`App`] with application name.
    pub fn new(name: impl AsRef<str>) -> sys::Result<Self> {
        #[cfg(feature = "compio-compat")]
        let runtime = RuntimeCompat::new(Runtime::new()?)?;
        let name = name.as_ref().to_string();
        #[allow(unused_mut)]
        let mut app = SysApp::new()?;
        #[cfg(not(any(windows, target_vendor = "apple")))]
        app.set_app_id(&name)?;
        Ok(Self {
            #[cfg(feature = "compio-compat")]
            runtime,
            app,
            name,
        })
    }

    /// The application name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Block on the future till it completes.
    ///
    /// The inner runtime might exits the inner application loop after the
    /// execution of the future.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        #[cfg(feature = "compio-compat")]
        {
            self.app.block_on(self.runtime.execute(future))
        }
        #[cfg(not(feature = "compio-compat"))]
        {
            self.app.block_on(future)
        }
    }

    /// Run the component till the first event is emitted. [`RunEvent`] is
    /// flattened to [`Result`].
    pub fn run<'a, T: Component>(
        &self,
        init: impl Into<T::Init<'a>>,
    ) -> Result<T::Event, T::Error> {
        self.block_on(async move {
            let mut component = Root::<T>::init(init).await?;
            let stream = component.run();
            let mut stream = std::pin::pin!(stream);
            stream
                .next()
                .await
                .expect("component exits unexpectedly")
                .flatten()
        })
    }

    /// Run the component utill [`RunEvent::Event`] is emitted. Other variants
    /// of [`RunEvent`] are ignored.
    pub fn run_until_event<'a, T: Component>(
        &self,
        init: impl Into<T::Init<'a>>,
    ) -> Result<T::Event, T::Error> {
        self.block_on(async move {
            let mut component = Root::<T>::init(init).await?;
            let stream = component.run();
            let mut stream = std::pin::pin!(stream);
            while let Some(event) = stream.next().await {
                match event {
                    RunEvent::Event(e) => return Ok(e),
                    RunEvent::UpdateErr(_e) => {
                        error!("Component update error: {_e:?}");
                    }
                    RunEvent::RenderErr(_e) => {
                        error!("Component render error: {_e:?}");
                    }
                    _ => {
                        warn!("Unrecognized event.");
                    }
                }
            }
            unreachable!("component exits unexpectedly")
        })
    }
}
