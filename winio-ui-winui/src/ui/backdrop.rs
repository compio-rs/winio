use std::cell::RefCell;

use windows::{
    UI::Color,
    core::{IInspectable_Vtbl, Ref, Result, imp::WeakRefCount, implement},
};
use winio_primitive::ColorTheme;
use winui3::{
    ChildClass, Compose, CreateInstanceFn,
    Microsoft::UI::{
        Composition::{
            ICompositionSupportsSystemBackdrop,
            SystemBackdrops::{DesktopAcrylicController, SystemBackdropConfiguration},
        },
        Xaml::{
            self as MUX,
            Media::{
                ISystemBackdropFactory, ISystemBackdropFactory_Vtbl, ISystemBackdropOverrides,
                ISystemBackdropOverrides_Impl, SystemBackdrop,
            },
        },
    },
};

use crate::color_theme;

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
}

impl CustomDesktopAcrylicBackdropControllerEntry {
    pub fn new(
        target: ICompositionSupportsSystemBackdrop,
        controller: DesktopAcrylicController,
        configuration: &SystemBackdropConfiguration,
    ) -> Result<Self> {
        controller.AddSystemBackdropTarget(&target)?;
        controller.SetSystemBackdropConfiguration(configuration)?;
        Ok(Self { target, controller })
    }
}

impl Drop for CustomDesktopAcrylicBackdropControllerEntry {
    fn drop(&mut self) {
        self.controller
            .RemoveSystemBackdropTarget(&self.target)
            .ok();
        self.controller.Close().ok();
    }
}

#[implement(ISystemBackdropOverrides, Agile = false)]
pub struct CustomDesktopAcrylicBackdrop {
    controllers: RefCell<Vec<CustomDesktopAcrylicBackdropControllerEntry>>,
}

impl CustomDesktopAcrylicBackdrop {
    pub fn compose() -> Result<SystemBackdrop> {
        Compose::compose(Self {
            controllers: RefCell::new(vec![]),
        })
    }
}

macro_rules! base {
    ($this:ident : $t:ty) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let base = winui3::Compose::<<Self as windows_core::IUnknownImpl>::Impl>::base($this);
            windows_core::Interface::cast::<$t>(base)
        }
    }};
    ($this:ident : $t:ty, $f:ident ($($args:expr),* $(,)?)) => {{
        #[allow(unused_unsafe)]
        unsafe {
            base!($this: $t).and_then(|b| {
                (windows_core::Interface::vtable(&b).$f)(windows_core::Interface::as_raw(&b), $(windows_core::Param::param($args).abi()),*).ok()
            })
        }
    }};
}

impl ISystemBackdropOverrides_Impl for CustomDesktopAcrylicBackdrop_Impl {
    fn OnTargetConnected(
        &self,
        target: Ref<ICompositionSupportsSystemBackdrop>,
        root: Ref<MUX::XamlRoot>,
    ) -> Result<()> {
        base!(
            self: ISystemBackdropOverrides,
            OnTargetConnected(target.as_ref(), root.as_ref())
        )?;

        let target = target.ok()?;
        let root = root.ok()?;

        let configuration =
            base!(self: SystemBackdrop)?.GetDefaultSystemBackdropConfiguration(target, root)?;
        let controller = DesktopAcrylicController::new()?;
        // Magic number to match Win32.
        controller.SetLuminosityOpacity(0.65)?;
        update_color(&controller)?;

        self.controllers
            .borrow_mut()
            .push(CustomDesktopAcrylicBackdropControllerEntry::new(
                target.clone(),
                controller,
                &configuration,
            )?);
        Ok(())
    }

    fn OnTargetDisconnected(&self, target: Ref<ICompositionSupportsSystemBackdrop>) -> Result<()> {
        base!(
            self: ISystemBackdropOverrides,
            OnTargetDisconnected(target.as_ref())
        )?;

        let target = target.ok()?;

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
        base!(
            self: ISystemBackdropOverrides,
            OnDefaultSystemBackdropConfigurationChanged(target.as_ref(), root.as_ref())
        )?;

        let target = target.ok()?;

        for entry in self.controllers.borrow().iter() {
            if entry.target == *target {
                update_color(&entry.controller)?;
                break;
            }
        }
        Ok(())
    }
}

impl ChildClass for CustomDesktopAcrylicBackdrop {
    type BaseType = SystemBackdrop;
    type FactoryInterface = ISystemBackdropFactory;

    fn create_interface_fn(vtable: &ISystemBackdropFactory_Vtbl) -> CreateInstanceFn {
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
