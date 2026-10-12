use inherit_methods_macro::inherit_methods;
use winio_handle::AsMenuContainer;

use crate::{
    Result,
    menu::{
        ActionClick, ActionHandle, RawPtr,
        ffi::{QMenu, menu_add_action},
    },
};

#[derive(Debug)]
pub struct CheckMenuItem {
    action: ActionHandle,
    click: ActionClick,
}

#[inherit_methods(from = "self.action")]
impl CheckMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let menu = parent.as_menu_container().as_qt::<QMenu>();
        let action = unsafe { menu_add_action(menu)? };
        unsafe {
            let mut action = RawPtr(action);
            action.pin_mut().setCheckable(true)?;
        }
        Ok(Self {
            action: ActionHandle::new(action),
            click: ActionClick::new(action)?,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.click.wait_click().await
    }
}

winio_handle::impl_as_menu_item!(CheckMenuItem, action);
