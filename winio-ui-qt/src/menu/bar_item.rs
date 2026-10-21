use std::fmt::Debug;

use inherit_methods_macro::inherit_methods;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{
    Result,
    menu::{
        ActionHandle, MenuBar, RawPtr,
        ffi::{QMenu, menu_bar_add_menu},
    },
};

#[derive(Debug)]
pub struct MenuBarItem {
    action: ActionHandle,
    menu: RawPtr<QMenu>,
}

#[inherit_methods(from = "self.action")]
impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        let menu = unsafe { menu_bar_add_menu(parent.menu_bar())? };
        let mut menu = RawPtr(menu);
        let action = unsafe { menu.pin_mut() }.menuAction()?;
        Ok(Self {
            action: ActionHandle::new(action),
            menu,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl AsMenuContainer for MenuBarItem {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        unsafe { BorrowedMenuContainer::qt(self.menu.0) }
    }
}
