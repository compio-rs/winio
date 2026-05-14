use std::{
    cell::RefCell,
    future::Future,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle},
    ptr::null,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use compio::driver::AsRawFd;
use compio_log::*;
use windows::{
    Foundation::Uri,
    core::{Array, HSTRING, IInspectable_Vtbl, Interface, Ref, h, imp::WeakRefCount, implement},
};
use windows_sys::Win32::{
    Foundation::{HWND, WAIT_FAILED, WAIT_OBJECT_0},
    System::Threading::{CreateEventW, INFINITE, SetEvent, WaitForMultipleObjects},
    UI::WindowsAndMessaging::MSG,
};
use winio_ui_windows_common::{PreferredAppMode, init_dark, set_preferred_app_mode};
use winui3::{
    ApartmentType, ChildClass, ChildClassImpl, Compose, CreateInstanceFn,
    Microsoft::UI::{
        Dispatching::{DispatcherQueue, DispatcherQueueHandler},
        Xaml::{
            Application, ApplicationInitializationCallback,
            ApplicationInitializationCallbackParams,
            Controls::XamlControlsResources,
            IApplicationFactory, IApplicationFactory_Vtbl, IApplicationOverrides,
            IApplicationOverrides_Impl, LaunchActivatedEventArgs,
            Markup::{
                IXamlMetadataProvider, IXamlMetadataProvider_Impl, IXamlType, XmlnsDefinition,
            },
            ResourceDictionary, UnhandledExceptionEventHandler,
            XamlTypeInfo::XamlControlsXamlMetaDataProvider,
        },
    },
    Windows::UI::Xaml::Interop::TypeName,
    bootstrap::{PackageDependency, WindowsAppSDKVersion},
    init_apartment,
};

use crate::{Error, Result};

pub(crate) struct WinUIApp {
    #[allow(dead_code)]
    winui_dependency: PackageDependency,
}

fn init_appsdk_with(
    vers: impl IntoIterator<Item = WindowsAppSDKVersion>,
) -> Result<PackageDependency> {
    for ver in vers {
        if let Ok(p) = PackageDependency::initialize_version(ver) {
            return Ok(p);
        }
    }
    PackageDependency::initialize()
}

impl WinUIApp {
    pub fn new() -> Result<Self> {
        init_apartment(ApartmentType::SingleThreaded)?;

        let winui_dependency = init_appsdk_with({
            use WindowsAppSDKVersion::*;
            [
                V2,
                V1_8,
                #[cfg(feature = "enable-cbs")]
                Cbs1_8,
                V1_7,
                V1_6,
                #[cfg(feature = "enable-cbs")]
                Cbs1_6,
                V1_5,
                #[cfg(feature = "enable-cbs")]
                Cbs,
                V1_4,
                V1_3,
                V1_2,
                #[cfg(not(feature = "media"))]
                V1_1,
                #[cfg(not(feature = "media"))]
                V1_0,
            ]
        })?;

        debug!("WinUI initialized: {winui_dependency:?}");

        init_dark();
        set_preferred_app_mode(PreferredAppMode::AllowDark);

        crate::hook::mrm::init_hook();
        if !crate::hook::mq::init_hook() {
            warn!("Message queue hooking failed, fallback to dedicated thread");
        }

        Ok(Self { winui_dependency })
    }
}

scoped_tls::scoped_thread_local!(pub(crate) static APP: WinUIApp);

fn app_start(_: Ref<'_, ApplicationInitializationCallbackParams>) -> Result<()> {
    debug!("Application::Start");

    let app = App::compose()?;
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

    APP.with(|app| {
        // spawn_runtime_thread()?;
        Result::Ok(())
    })?;

    Ok(())
}

static THREAD_COUNTER: AtomicBool = AtomicBool::new(false);

struct ThreadGuard;

impl ThreadGuard {
    fn new() -> Option<Self> {
        if THREAD_COUNTER
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            Some(Self)
        } else {
            None
        }
    }
}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        THREAD_COUNTER.store(false, Ordering::Release);
        info!("Runtime thread exited");
    }
}

fn spawn_runtime_thread() -> Result<()> {
    if let Some(guard) = ThreadGuard::new() {
        let dispatcher = DispatcherQueue::GetForCurrentThread()?;
        compio::runtime::spawn_blocking(move || {
            let _guard = guard;
            loop {
                let is_ready =
                    resume_foreground(&dispatcher, move || winio_pollable::run_current_task());
                if is_ready == Some(true) {
                    break;
                }
            }
        })
        .detach();
    }
    Ok(())
}

fn resume_foreground<T: Send + 'static>(
    dispatcher: &DispatcherQueue,
    f: impl (Fn() -> T) + Send + 'static,
) -> Option<T> {
    let (tx, rx) = oneshot::channel();
    let tx = RefCell::new(Some(tx));
    let queued = dispatcher
        .TryEnqueue(&DispatcherQueueHandler::new(move || {
            if let Some(tx) = tx.borrow_mut().take() {
                tx.send(f()).ok();
            }
            Ok(())
        }))
        .unwrap_or_default();
    if queued { rx.recv().ok() } else { None }
}

#[implement(IApplicationOverrides, IXamlMetadataProvider)]
struct App {
    provider: XamlControlsXamlMetaDataProvider,
}

impl App {
    pub(crate) fn compose() -> Result<Application> {
        Compose::compose(Self {
            provider: XamlControlsXamlMetaDataProvider::new()?,
        })
    }
}

impl ChildClassImpl for App_Impl {}

impl IApplicationOverrides_Impl for App_Impl {
    fn OnLaunched(&self, _: Ref<LaunchActivatedEventArgs>) -> Result<()> {
        debug!("App::OnLaunched");

        let resources = self.base()?.cast::<Application>()?.Resources()?;
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

impl IXamlMetadataProvider_Impl for App_Impl {
    fn GetXamlType(&self, ty: &TypeName) -> Result<IXamlType> {
        self.provider.GetXamlType(ty)
    }

    fn GetXamlTypeByFullName(&self, name: &HSTRING) -> Result<IXamlType> {
        self.provider.GetXamlTypeByFullName(name)
    }

    fn GetXmlnsDefinitions(&self) -> Result<Array<XmlnsDefinition>> {
        self.provider.GetXmlnsDefinitions()
    }
}

impl ChildClass for App {
    type BaseType = Application;
    type FactoryInterface = IApplicationFactory;

    fn create_interface_fn(vtable: &IApplicationFactory_Vtbl) -> CreateInstanceFn {
        vtable.CreateInstance
    }

    fn identity_vtable(vtable: &mut Self::Outer) -> &mut &'static IInspectable_Vtbl {
        &mut vtable.identity
    }

    fn ref_count(vtable: &Self::Outer) -> &WeakRefCount {
        &vtable.count
    }

    fn into_outer(self) -> Self::Outer {
        Self::into_outer(self)
    }
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    let app = WinUIApp::new().expect("Failed to initialize WinUI");

    let waker =
        winio_ui_windows_common::waker().expect("failed to create waker for current thread");

    APP.set(&app, || {
        let result = RefCell::new(None);
        let future = async {
            let res = future.await;
            Application::Current()
                .expect("Failed to get current application")
                .Exit()
                .expect("Failed to exit application");
            result.replace(Some(res));
        };
        winio_pollable::enter_block_on(future, waker, || {
            Application::Start(&ApplicationInitializationCallback::new(app_start))
                .expect("Failed to start application");

            result.take().expect("Application exits but no result")
        })
    })
}
