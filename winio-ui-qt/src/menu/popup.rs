use inherit_methods_macro::inherit_methods;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{
    Result,
    menu::{
        ActionHandle, RawPtr,
        ffi::{QMenu, menu_add_menu},
    },
};

#[derive(Debug)]
pub struct PopupMenuItem {
    action: ActionHandle,
    menu: *mut QMenu,
}

#[inherit_methods(from = "self.action")]
impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let menu = parent.as_menu_container().as_qt::<QMenu>();
        let submenu = unsafe { menu_add_menu(menu) }?;
        let action = unsafe {
            let mut submenu = RawPtr(submenu);
            submenu.pin_mut().menuAction()?
        };
        Ok(Self {
            action: ActionHandle::new(action),
            menu: submenu,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl AsMenuContainer for PopupMenuItem {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        unsafe { BorrowedMenuContainer::qt::<QMenu>(self.menu) }
    }
}

winio_handle::impl_as_menu_item!(PopupMenuItem, action);
