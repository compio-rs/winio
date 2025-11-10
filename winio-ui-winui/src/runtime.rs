use std::{cell::OnceCell, future::Future, time::Duration};

use compio::driver::AsRawFd;
use compio_log::*;
use windows::{
    Foundation::Uri,
    Win32::Graphics::Direct2D::{
        D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1Factory2,
    },
    core::{
        Array, HSTRING, IInspectable_Vtbl, Interface, Ref, Result, h, imp::WeakRefCount, implement,
    },
};
use windows_sys::Win32::{
    Foundation::{HWND, WAIT_FAILED, WAIT_OBJECT_0},
    System::Threading::INFINITE,
    UI::WindowsAndMessaging::{
        MSG, MWMO_ALERTABLE, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_REMOVE,
        PeekMessageW, QS_ALLINPUT, WM_QUIT,
    },
};
use winio_ui_windows_common::{PreferredAppMode, init_dark, set_preferred_app_mode};
use winui3::{
    ApartmentType, ChildClass, ChildClassImpl, Compose, CreateInstanceFn,
    Microsoft::UI::Xaml::{
        Application, ApplicationInitializationCallback, ApplicationInitializationCallbackParams,
        Controls::XamlControlsResources,
        IApplicationFactory, IApplicationFactory_Vtbl, IApplicationOverrides,
        IApplicationOverrides_Impl, LaunchActivatedEventArgs,
        Markup::{IXamlMetadataProvider, IXamlMetadataProvider_Impl, IXamlType, XmlnsDefinition},
        ResourceDictionary, UnhandledExceptionEventHandler,
        XamlTypeInfo::XamlControlsXamlMetaDataProvider,
    },
    Windows::UI::Xaml::Interop::TypeName,
    bootstrap::{PackageDependency, WindowsAppSDKVersion},
    init_apartment,
};

use crate::RUNTIME;

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    #[allow(dead_code)]
    winui_dependency: PackageDependency,
    d2d1: OnceCell<ID2D1Factory2>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn new() -> Self {
        init_apartment(ApartmentType::SingleThreaded).unwrap();

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
        })
        .unwrap();

        debug!("WinUI initialized: {winui_dependency:?}");

        init_dark();
        set_preferred_app_mode(PreferredAppMode::AllowDark);

        crate::hook::mrm::init_hook();

        let runtime = compio::runtime::Runtime::new().unwrap();

        Self {
            runtime,
            winui_dependency,
            d2d1: OnceCell::new(),
        }
    }

    pub(crate) fn d2d1(&self) -> &ID2D1Factory2 {
        self.d2d1.get_or_init(|| unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap()
        })
    }

    pub(crate) fn run(&self) -> bool {
        self.runtime.run()
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let _guard = crate::hook::mq::HookGuard::new().unwrap();
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

    Ok(())
}

pub(crate) fn run_runtime(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32 {
    RUNTIME.with(|runtime| {
        loop {
            runtime.runtime.poll_with(Some(Duration::ZERO));
            let remaining_tasks = runtime.run();
            let timeout = if remaining_tasks {
                Some(Duration::ZERO)
            } else {
                runtime.runtime.current_timeout()
            };
            debug!("waiting in {timeout:?}");
            let timeout = match timeout {
                Some(timeout) => timeout.as_millis() as u32,
                None => INFINITE,
            };
            let handle = runtime.runtime.as_raw_fd();
            let res = unsafe {
                MsgWaitForMultipleObjectsEx(
                    1,
                    &handle,
                    timeout,
                    QS_ALLINPUT,
                    MWMO_ALERTABLE | MWMO_INPUTAVAILABLE,
                )
            };
            const WAIT_OBJECT_1: u32 = WAIT_OBJECT_0 + 1;
            match res {
                WAIT_OBJECT_1 => {
                    let res = unsafe { PeekMessageW(msg, hwnd, min, max, PM_REMOVE) };
                    if res != 0 {
                        if unsafe { (*msg).message } == WM_QUIT {
                            return 0;
                        } else {
                            return 1;
                        }
                    }
                }
                WAIT_FAILED => return -1,
                _ => {}
            }
        }
    })
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
