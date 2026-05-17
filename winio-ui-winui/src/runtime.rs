use std::{
    cell::RefCell,
    future::Future,
    sync::{Arc, Mutex},
    task::{Wake, Waker},
};

use compio_log::*;
use futures_util::FutureExt;
use windows::{
    Foundation::Uri,
    core::{Array, HSTRING, IInspectable_Vtbl, Interface, Ref, h, imp::WeakRefCount, implement},
};
use winio_ui_windows_common::{PreferredAppMode, init_dark, set_preferred_app_mode};
use winui3::{
    ApartmentType, ChildClass, ChildClassImpl, Compose, CreateInstanceFn,
    Microsoft::UI::{
        Dispatching::{DispatcherQueue, DispatcherQueueHandler},
        Xaml::{
            Application, ApplicationInitializationCallback,
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

use crate::Result;

pub struct App {
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

impl App {
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

        Ok(Self { winui_dependency })
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let dispatcher = Arc::new(DispatcherWaker::new());
        let waker = Waker::from(dispatcher.clone());

        let result = RefCell::new(None);
        let future = future.map(|res| {
            Application::Current()
                .expect("Failed to get current application")
                .Exit()
                .expect("Failed to exit application");
            result.replace(Some(res));
        });
        winio_pollable::enter_block_on(future, waker, || {
            let dispatcher = RefCell::new(Some(dispatcher));
            Application::Start(&ApplicationInitializationCallback::new(move |_| {
                app_start(dispatcher.borrow_mut().take().unwrap())
            }))
            .expect("Failed to start application");

            result.take().expect("Application exits but no result")
        })
    }
}

fn app_start(waker: Arc<DispatcherWaker>) -> Result<()> {
    debug!("Application::Start");

    let app = XamlApp::compose()?;
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

    let dispatcher = DispatcherQueue::GetForCurrentThread()?;
    waker.dispatcher.lock().unwrap().replace(dispatcher);
    waker.wake();

    Ok(())
}

#[implement(IApplicationOverrides, IXamlMetadataProvider)]
struct XamlApp {
    provider: XamlControlsXamlMetaDataProvider,
}

impl XamlApp {
    pub(crate) fn compose() -> Result<Application> {
        Compose::compose(Self {
            provider: XamlControlsXamlMetaDataProvider::new()?,
        })
    }
}

impl ChildClassImpl for XamlApp_Impl {}

impl IApplicationOverrides_Impl for XamlApp_Impl {
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

impl IXamlMetadataProvider_Impl for XamlApp_Impl {
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

impl ChildClass for XamlApp {
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

struct DispatcherWaker {
    dispatcher: Mutex<Option<DispatcherQueue>>,
}

impl DispatcherWaker {
    pub fn new() -> Self {
        Self {
            dispatcher: Mutex::new(None),
        }
    }

    fn wake_impl(&self) {
        let dispatcher = self.dispatcher.lock().unwrap();
        if let Some(dispatcher) = dispatcher.as_ref() {
            dispatcher
                .TryEnqueue(&DispatcherQueueHandler::new(|| {
                    winio_pollable::run_current_task();
                    Ok(())
                }))
                .ok();
        }
    }
}

impl Wake for DispatcherWaker {
    fn wake(self: Arc<Self>) {
        self.wake_impl();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_impl();
    }
}
