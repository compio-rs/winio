use {crate::sys::Runtime, std::future::Future, winio_elm::Component};

#[cfg(target_os = "android")]
pub use crate::sys::AndroidApp;

/// Root application, manages the async runtime.
pub struct App {
    runtime: Runtime,
    name: Option<String>,
}

impl App {
    /// Create [`App`] with application name.
    pub fn new(name: impl AsRef<str>) -> Self {
        #[allow(unused_mut)]
        let mut runtime = Runtime::new();
        let name = name.as_ref().to_string();
        #[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
        runtime.set_app_id(&name);
        Self {
            runtime,
            name: Some(name),
        }
    }

    /// The application name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set android activity instance
    #[cfg(target_os = "android")]
    pub fn set_android_app(&mut self, android_app: AndroidApp) -> &mut Self {
        self.runtime.set_android_app(android_app);
        self
    }

    /// Block on the future till it completes.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Create and manage the component, till it posts an event. The application
    /// returns the first event from the component.
    pub fn run<'a, T: Component>(&mut self, init: impl Into<T::Init<'a>>) -> T::Event {
        self.block_on(winio_elm::run::<T>(init))
    }
}
