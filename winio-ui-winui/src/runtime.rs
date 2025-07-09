use std::{future::Future, time::Duration};

use compio::driver::AsRawFd;
use compio_log::*;
use windows::{
    Foundation::Uri,
    core::{Ref, Result, h},
};
use windows_sys::Win32::System::Threading::{INFINITE, WaitForSingleObject};
use winui3::{
    ApartmentType,
    Microsoft::UI::{
        Dispatching::{DispatcherQueue, DispatcherQueueHandler},
        Xaml::{
            Application, ApplicationInitializationCallback,
            ApplicationInitializationCallbackParams, Controls::XamlControlsResources,
            LaunchActivatedEventArgs, ResourceDictionary, UnhandledExceptionEventHandler,
        },
    },
    XamlApp, XamlAppOverrides,
    bootstrap::PackageDependency,
    init_apartment,
};

use crate::RUNTIME;

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    #[allow(dead_code)]
    winui_dependency: PackageDependency,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        init_apartment(ApartmentType::SingleThreaded).unwrap();

        let winui_dependency = winui3::bootstrap::PackageDependency::initialize().unwrap();

        debug!("WinUI initialized: {winui_dependency:?}");

        let runtime = compio::runtime::Runtime::new().unwrap();

        Self {
            runtime,
            winui_dependency,
        }
    }

    pub(crate) fn run(&self) -> bool {
        self.runtime.run()
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let mut result = None;
            unsafe {
                self.runtime.spawn_unchecked(async {
                    result = Some(future.await);
                    Application::Current().unwrap().Exit().unwrap();
                })
            }
            .detach();

            Application::Start(&ApplicationInitializationCallback::new(app_start)).unwrap();

            result.unwrap()
        })
    }
}

fn resume_foreground<T: Send + 'static>(
    dispatcher: &DispatcherQueue,
    mut f: impl (FnMut() -> T) + Send + 'static,
) -> Option<T> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    dispatcher
        .TryEnqueue(&DispatcherQueueHandler::new(move || {
            tx.send(f()).unwrap();
            Ok(())
        }))
        .unwrap();
    rx.recv().ok()
}

fn app_start(_: Ref<'_, ApplicationInitializationCallbackParams>) -> Result<()> {
    debug!("Application::Start");

    let app = App::create()?;
    app.UnhandledException(Some(&UnhandledExceptionEventHandler::new(
        |_sender, args| {
            #[allow(clippy::single_match)]
            match args.as_ref() {
                #[allow(unused)]
                Some(args) => {
                    error!("Unhandled exception: {}", args.Exception()?);
                    error!("{}", args.Message()?);
                }
                None => {
                    error!("Unhandled exception occurred");
                }
            }
            Ok(())
        },
    )))?;

    let dispatcher = DispatcherQueue::GetForCurrentThread().unwrap();
    let handle = RUNTIME.with(|runtime| runtime.runtime.as_raw_fd());

    std::thread::spawn(move || {
        loop {
            let timeout = resume_foreground(&dispatcher, {
                move || {
                    RUNTIME.with(|runtime| {
                        runtime.runtime.poll_with(Some(Duration::ZERO));
                        let remaining_tasks = runtime.run();
                        if remaining_tasks {
                            Some(Duration::ZERO)
                        } else {
                            runtime.runtime.current_timeout()
                        }
                    })
                }
            });
            let Some(timeout) = timeout else {
                break;
            };
            let timeout = match timeout {
                Some(timeout) => timeout.as_millis() as u32,
                None => INFINITE,
            };
            debug!("before WaitForSingleObject");
            unsafe {
                WaitForSingleObject(handle as _, timeout);
            }
            debug!("after WaitForSingleObject");
        }
    });

    Ok(())
}

struct App {}

impl App {
    pub(crate) fn create() -> Result<Application> {
        let app = Self {};
        XamlApp::compose(app)
    }
}

impl XamlAppOverrides for App {
    fn OnLaunched(&self, base: &Application, _: Option<&LaunchActivatedEventArgs>) -> Result<()> {
        debug!("App::OnLaunched");

        let resources = base.Resources()?;
        let merged_dictionaries = resources.MergedDictionaries()?;
        let xaml_controls_resources = XamlControlsResources::new()?;
        merged_dictionaries.Append(&xaml_controls_resources)?;

        let compact_resources = ResourceDictionary::new()?;
        compact_resources.SetSource(&Uri::CreateUri(h!(
            "ms-appx:///Microsoft.UI.Xaml/DensityStyles/Compact.xaml"
        ))?)?;
        merged_dictionaries.Append(&compact_resources)?;

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        debug!("App::drop");
    }
}
