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
    /// Create [`App`] with application name.
    pub fn new(
        name: impl AsRef<str>,
        #[cfg(target_os = "android")] app: android_activity::AndroidApp,
    ) -> sys::Result<Self> {
        Self::new_impl(
            name,
            #[cfg(target_os = "android")]
            app,
        )
    }

    fn new_impl(
        name: impl AsRef<str>,
        #[cfg(target_os = "android")] app: android_activity::AndroidApp,
    ) -> sys::Result<Self> {
        let name = name.as_ref().to_string();
        #[allow(unused_mut)]
        let mut app = SysApp::new(
            #[cfg(target_os = "android")]
            app,
        )?;
        #[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
        app.set_app_id(&name)?;
        Ok(Self { app, name })
    }

    /// The application name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Run the component till the first event is emitted. [`RunEvent`] is
    /// flattened to [`Result`].
    pub async fn execute<'a, T: Component>(
        init: impl Into<T::Init<'a>>,
    ) -> Result<T::Event, T::Error> {
        let mut component = Root::<T>::init(init).await?;
        let stream = component.run();
        let mut stream = std::pin::pin!(stream);
        stream
            .next()
            .await
            .expect("component exits unexpectedly")
            .flatten()
    }

    /// Run the component utill [`RunEvent::Event`] is emitted. Other variants
    /// of [`RunEvent`] are ignored.
    pub async fn execute_until_event<'a, T: Component>(
        init: impl Into<T::Init<'a>>,
    ) -> Result<T::Event, T::Error> {
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
    }
}

#[cfg(not(target_os = "android"))]
impl App {
    /// Block on the future till it completes.
    ///
    /// The inner runtime might exits the inner application loop after the
    /// execution of the future.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let future = self.runtime.execute(future);
        if std::mem::size_of_val(&future) >= 2048 {
            self.app.block_on(Box::pin(future))
        } else {
            self.app.block_on(future)
        }
    }

    /// Equivalent to calling [`block_on`](Self::block_on) on
    /// [`execute`](Self::execute).
    pub fn run<'a, T: Component>(
        &self,
        init: impl Into<T::Init<'a>>,
    ) -> Result<T::Event, T::Error> {
        self.block_on(Self::execute::<T>(init))
    }

    /// Equivalent to calling [`block_on`](Self::block_on) on
    /// [`execute_until_event`](Self::execute_until_event).
    pub fn run_until_event<'a, T: Component>(
        &self,
        init: impl Into<T::Init<'a>>,
    ) -> Result<T::Event, T::Error> {
        self.block_on(Self::execute_until_event::<T>(init))
    }
}

#[cfg(target_os = "android")]
impl App {
    /// Block on the future till it completes.
    ///
    /// The inner runtime might exits the inner application loop after the
    /// execution of the future.
    ///
    /// The future is created on the main thread instead of the current thread.
    pub fn block_on<F: Future<Output = ()>>(
        &self,
        future: impl (FnOnce() -> F) + Sync + Send + 'static,
    ) {
        self.block_on_with_runtime(|| Runtime::new().unwrap(), future);
    }

    /// Block on the future till it completes with a custom runtime.
    ///
    /// Both the runtime and the future is created on the main thread instead of
    /// the current thread.
    pub fn block_on_with_runtime<F: Future<Output = ()>>(
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
}
