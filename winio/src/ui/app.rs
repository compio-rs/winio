use std::future::Future;

use compio_log::{error, warn};
use futures_util::StreamExt;
use winio_elm::{Component, Root, RunEvent};

use crate::{sys, sys::App as SysApp};

#[cfg(feature = "compio-compat")]
type WinioRuntimeCompat = compio::compat::RuntimeCompat<sys::CompioAdapter>;
#[cfg(feature = "compio-compat")]
use compio::runtime::Runtime;

#[cfg(not(feature = "compio-compat"))]
mod compat_stub {
    use std::io;

    pub struct Runtime(());

    impl Runtime {
        #[inline]
        pub fn new() -> io::Result<Self> {
            Ok(Self(()))
        }
    }

    pub struct WinioRuntimeCompat(());

    impl WinioRuntimeCompat {
        #[inline]
        pub fn new(_runtime: Runtime) -> io::Result<Self> {
            Ok(Self(()))
        }

        #[inline]
        pub fn execute<F: Future>(&self, f: F) -> F {
            f
        }
    }
}

#[cfg(not(feature = "compio-compat"))]
use compat_stub::*;

/// Root application, manages the async runtime.
pub struct App {
    app: SysApp,
    name: String,
}

impl App {
    /// Create [`AppBuilder`] to build the application.
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// The application name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Builder for [`App`].
#[derive(Default)]
pub struct AppBuilder {
    name: String,
    #[cfg(target_os = "android")]
    app: Option<android_activity::AndroidApp>,
}

impl AppBuilder {
    /// Set the application name. The name is used for application
    /// identification and might be used for notification or icon display on
    /// some platforms.
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    /// Set the Android application. This is required on Android platform.
    #[cfg(target_os = "android")]
    pub fn android_app(mut self, app: android_activity::AndroidApp) -> Self {
        self.app = Some(app);
        self
    }

    /// Build the application. This will consume the builder and return the
    /// application instance.
    pub fn build(self) -> sys::Result<App> {
        #[allow(unused_mut)]
        let mut app = SysApp::new(
            #[cfg(target_os = "android")]
            self.app.ok_or(sys::Error::NoApp)?,
        )?;
        #[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
        app.set_app_id(&self.name)?;
        Ok(App {
            app,
            name: self.name,
        })
    }
}

#[cfg(not(target_os = "android"))]
impl App {
    /// Block on the future till it completes.
    ///
    /// The inner runtime might exits the inner application loop after the
    /// execution of the future.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.block_on_with_runtime_impl(Runtime::new().unwrap(), future)
    }

    fn block_on_with_runtime_impl<F: Future>(&self, runtime: Runtime, future: F) -> F::Output {
        let compat = WinioRuntimeCompat::new(runtime).unwrap();
        let future = compat.execute(future);
        if std::mem::size_of_val(&future) >= 2048 {
            self.app.block_on(Box::pin(future))
        } else {
            self.app.block_on(future)
        }
    }

    /// Block on the future with a custom runtime.
    ///
    /// The inner runtime might exits the inner application loop after the
    /// execution of the future.
    #[cfg(feature = "compio-compat")]
    pub fn block_on_with_runtime<F: Future>(&self, runtime: Runtime, future: F) -> F::Output {
        self.block_on_with_runtime_impl(runtime, future)
    }
}

#[cfg(target_os = "android")]
impl App {
    /// Spawn the future on the main thread.
    ///
    /// The inner runtime might exits the inner application loop after the
    /// execution of the future.
    pub fn spawn<F: Future<Output = ()>>(
        &self,
        future: impl (FnOnce() -> F) + Sync + Send + 'static,
    ) {
        self.spawn_with_runtime_impl(|| Runtime::new().unwrap(), future);
    }

    fn spawn_with_runtime_impl<F: Future<Output = ()>>(
        &self,
        runtime: impl (FnOnce() -> Runtime) + Sync + Send + 'static,
        future: impl (FnOnce() -> F) + Sync + Send + 'static,
    ) {
        self.app.block_on(move || {
            let runtime = runtime();
            let compat = WinioRuntimeCompat::new(runtime).unwrap();
            async move { compat.execute(future()).await }
        })
    }

    #[cfg(feature = "compio-compat")]
    /// Spawn the future on the main thread with a custom runtime.
    pub fn spawn_with_runtime<F: Future<Output = ()>>(
        &self,
        runtime: impl (FnOnce() -> Runtime) + Sync + Send + 'static,
        future: impl (FnOnce() -> F) + Sync + Send + 'static,
    ) {
        self.spawn_with_runtime_impl(runtime, future);
    }
}

/// Extension trait for [`Component`] to run the component.
#[allow(async_fn_in_trait)]
pub trait ComponentExt: Component {
    /// Run the component till the first event is emitted. [`RunEvent`] is
    /// flattened to [`Result`].
    async fn run<'a>(init: impl Into<Self::Init<'a>>) -> Result<Self::Event, Self::Error>;

    /// Run the component utill [`RunEvent::Event`] is emitted. Other variants
    /// of [`RunEvent`] are ignored.
    async fn run_until_event<'a>(
        init: impl Into<Self::Init<'a>>,
    ) -> Result<Self::Event, Self::Error>;
}

impl<T: Component> ComponentExt for T {
    async fn run<'a>(init: impl Into<Self::Init<'a>>) -> Result<Self::Event, Self::Error> {
        let mut component = Root::<Self>::init(init).await?;
        let stream = component.run();
        let mut stream = std::pin::pin!(stream);
        stream
            .next()
            .await
            .expect("component exits unexpectedly")
            .flatten()
    }

    async fn run_until_event<'a>(
        init: impl Into<Self::Init<'a>>,
    ) -> Result<Self::Event, Self::Error> {
        let mut component = Root::<Self>::init(init).await?;
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
    }
}
