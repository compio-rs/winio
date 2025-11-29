use std::{
    cell::RefCell,
    future::Future,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle},
    ptr::null,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use compio::driver::AsRawFd;
use compio_log::*;
use windows::{
    Foundation::Uri,
    Win32::Graphics::Direct2D::ID2D1Factory2,
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

use crate::{Error, RUNTIME, Result};

pub struct Runtime {
    runtime: winio_ui_windows_common::Runtime,
    #[allow(dead_code)]
    winui_dependency: PackageDependency,
    shutdown: Option<Arc<OwnedHandle>>,
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

impl Runtime {
    pub fn new() -> Result<Self> {
        init_apartment(ApartmentType::SingleThreaded)?;

        let winui_dependency = init_appsdk_with({
            use WindowsAppSDKVersion::*;
            [
                V1_8,
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
        let shutdown = if !crate::hook::mq::init_hook() {
            warn!("Message queue hooking failed, fallback to dedicated thread");
            Some(Arc::new(unsafe {
                OwnedHandle::from_raw_handle({
                    let handle = CreateEventW(null(), 0, 0, null());
                    if handle.is_null() {
                        return Err(Error::from_thread());
                    }
                    handle
                })
            }))
        } else {
            None
        };

        let runtime = winio_ui_windows_common::Runtime::new()?;

        Ok(Self {
            runtime,
            winui_dependency,
            shutdown,
        })
    }

    pub(crate) fn d2d1(&self) -> Result<&ID2D1Factory2> {
        self.runtime.d2d1()
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
                    Application::Current()
                        .expect("Failed to get current application")
                        .Exit()
                        .expect("Failed to exit application");
                })
            }
            .detach();

            Application::Start(&ApplicationInitializationCallback::new(app_start))
                .expect("Failed to start application");

            if let Some(shutdown) = &self.shutdown {
                unsafe {
                    SetEvent(shutdown.as_raw_handle());
                }
            }

            result.expect("Application exits but no result")
        })
    }
}

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

    RUNTIME.with(|runtime| {
        if let Some(shutdown) = runtime.shutdown.clone() {
            spawn_runtime_thread(runtime.runtime.as_raw_fd() as *const _ as usize, shutdown)?;
        }
        Result::Ok(())
    })?;

    Ok(())
}

pub(crate) unsafe fn run_runtime(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> Option<i32> {
    if RUNTIME.is_set() {
        let res =
            RUNTIME.with(|runtime| unsafe { runtime.runtime.get_message(msg, hwnd, min, max) });
        Some(res)
    } else {
        None
    }
}

static THREAD_COUNTER: AtomicU8 = AtomicU8::new(0);

fn spawn_runtime_thread(runtime: usize, shutdown: Arc<OwnedHandle>) -> Result<()> {
    if THREAD_COUNTER
        .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        let dispatcher = DispatcherQueue::GetForCurrentThread()?;
        compio::runtime::spawn_blocking(move || {
            loop {
                let timeout = resume_foreground(&dispatcher, {
                    move || RUNTIME.with(|runtime| runtime.runtime.poll_and_run())
                });
                let Some(timeout) = timeout else {
                    break;
                };
                debug!("Waiting in {timeout:?}");
                let timeout = match timeout {
                    Some(timeout) => timeout.as_millis() as u32,
                    None => INFINITE,
                };
                let handles = [shutdown.as_raw_handle(), runtime as RawHandle];
                let res = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, timeout) };
                if res == WAIT_OBJECT_0 {
                    break;
                } else if res == WAIT_FAILED {
                    error!("WaitForMultipleObjects: {:?}", Error::from_thread());
                    break;
                }
            }
            THREAD_COUNTER.store(0, Ordering::Release);
            info!("Runtime thread exited");
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
