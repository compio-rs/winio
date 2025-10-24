use std::cell::RefCell;

use windows::{
    Foundation::TypedEventHandler,
    UI::{Color, ViewManagement::UISettings},
    core::{IInspectable_Vtbl, Interface, Ref, Result, imp::WeakRefCount, implement},
};
use winio_primitive::ColorTheme;
use winui3::{
    ChildClass, Compose, CreateInstanceFn,
    Microsoft::UI::{
        Composition::{
            ICompositionSupportsSystemBackdrop,
            SystemBackdrops::{DesktopAcrylicController, SystemBackdropConfiguration},
        },
        Dispatching::{DispatcherQueue, DispatcherQueueController, DispatcherQueueHandler},
        Xaml::{
            self as MUX,
            Media::{
                ISystemBackdropFactory, ISystemBackdropOverrides, ISystemBackdropOverrides_Impl,
                SystemBackdrop,
            },
        },
    },
};

use crate::color_theme;

fn ensure_windows_system_dispatcher_controller() -> Result<Option<DispatcherQueueController>> {
    if DispatcherQueue::GetForCurrentThread().is_err() {
        return Ok(Some(DispatcherQueueController::CreateOnCurrentThread()?));
    }
    Ok(None)
}

// Magic colors to match Win32.
const fn color(dark: bool) -> Color {
    if dark {
        Color {
            A: 0xFF,
            R: 0x54,
            G: 0x54,
            B: 0x54,
        }
    } else {
        Color {
            A: 0xFF,
            R: 0xD3,
            G: 0xD3,
            B: 0xD3,
        }
    }
}

fn update_color(controller: &DesktopAcrylicController) -> Result<()> {
    let color = color(color_theme() == ColorTheme::Dark);
    controller.SetTintColor(color)?;
    controller.SetFallbackColor(color)?;
    Ok(())
}

struct CustomDesktopAcrylicBackdropControllerEntry {
    controller: DesktopAcrylicController,
    target: ICompositionSupportsSystemBackdrop,
    settings: UISettings,
    token: i64,
}

impl CustomDesktopAcrylicBackdropControllerEntry {
    pub fn new(
        dispatcher: DispatcherQueue,
        target: ICompositionSupportsSystemBackdrop,
        controller: DesktopAcrylicController,
        configuration: &SystemBackdropConfiguration,
        settings: UISettings,
    ) -> Result<Self> {
        let token = {
            let controller = controller.clone();
            settings.ColorValuesChanged(&TypedEventHandler::new(move |_, _| {
                let controller = controller.clone();
                dispatcher.TryEnqueue(&DispatcherQueueHandler::new(move || {
                    update_color(&controller)
                }))?;
                Ok(())
            }))?
        };

        controller.AddSystemBackdropTarget(&target)?;
        controller.SetSystemBackdropConfiguration(configuration)?;
        Ok(Self {
            target,
            controller,
            settings,
            token,
        })
    }
}

impl Drop for CustomDesktopAcrylicBackdropControllerEntry {
    fn drop(&mut self) {
        self.controller
            .RemoveSystemBackdropTarget(&self.target)
            .ok();
        self.controller.Close().ok();
        self.settings.RemoveColorValuesChanged(self.token).ok();
    }
}

#[implement(ISystemBackdropOverrides)]
pub struct CustomDesktopAcrylicBackdrop {
    settings: UISettings,
    controllers: RefCell<Vec<CustomDesktopAcrylicBackdropControllerEntry>>,
    dispatcher: Option<DispatcherQueueController>,
}

impl CustomDesktopAcrylicBackdrop {
    pub fn compose() -> Result<SystemBackdrop> {
        let dispatcher = ensure_windows_system_dispatcher_controller()?;
        Compose::compose(Self {
            settings: UISettings::new()?,
            dispatcher,
            controllers: RefCell::new(vec![]),
        })
    }
}

impl ISystemBackdropOverrides_Impl for CustomDesktopAcrylicBackdrop_Impl {
    fn OnTargetConnected(
        &self,
        target: Ref<ICompositionSupportsSystemBackdrop>,
        root: Ref<MUX::XamlRoot>,
    ) -> Result<()> {
        let base = unsafe { Compose::<CustomDesktopAcrylicBackdrop>::base(self) };
        let target = target.ok()?;
        let root = root.ok()?;

        unsafe {
            let base = base.cast::<ISystemBackdropOverrides>()?;
            (base.vtable().OnTargetConnected)(base.as_raw(), target.as_raw(), root.as_raw())
                .ok()?;
        }

        let configuration = base
            .cast::<SystemBackdrop>()?
            .GetDefaultSystemBackdropConfiguration(target, root)?;
        let controller = DesktopAcrylicController::new()?;
        // Magic number to match Win32.
        controller.SetLuminosityOpacity(0.65)?;
        update_color(&controller)?;

        let dispatcher = if let Some(controller) = &self.dispatcher {
            controller.DispatcherQueue()?
        } else {
            DispatcherQueue::GetForCurrentThread()?
        };
        self.controllers
            .borrow_mut()
            .push(CustomDesktopAcrylicBackdropControllerEntry::new(
                dispatcher,
                target.clone(),
                controller,
                &configuration,
                self.settings.clone(),
            )?);
        Ok(())
    }

    fn OnTargetDisconnected(&self, target: Ref<ICompositionSupportsSystemBackdrop>) -> Result<()> {
        let base = unsafe { Compose::<CustomDesktopAcrylicBackdrop>::base(self) };
        let target = target.ok()?;
        unsafe {
            let base = base.cast::<ISystemBackdropOverrides>()?;
            (base.vtable().OnTargetDisconnected)(base.as_raw(), target.as_raw()).ok()?;
        }
        let mut controllers = self.controllers.borrow_mut();
        if let Some(pos) = controllers.iter().position(|entry| entry.target == *target) {
            controllers.remove(pos);
        }
        Ok(())
    }

    fn OnDefaultSystemBackdropConfigurationChanged(
        &self,
        target: Ref<ICompositionSupportsSystemBackdrop>,
        root: Ref<MUX::XamlRoot>,
    ) -> Result<()> {
        unsafe {
            let base = Compose::<CustomDesktopAcrylicBackdrop>::base(self)
                .cast::<ISystemBackdropOverrides>()?;
            let target = target.ok()?;
            let root = root.ok()?;
            (base.vtable().OnDefaultSystemBackdropConfigurationChanged)(
                base.as_raw(),
                target.as_raw(),
                root.as_raw(),
            )
            .ok()
        }
    }
}

impl ChildClass for CustomDesktopAcrylicBackdrop {
    type BaseType = SystemBackdrop;
    type FactoryInterface = ISystemBackdropFactory;

    fn create_interface_fn(
        vtable: &<Self::FactoryInterface as Interface>::Vtable,
    ) -> CreateInstanceFn {
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
