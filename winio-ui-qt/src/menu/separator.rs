use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};

use crate::{
    Result,
    menu::{ActionHandle, RawPtr, ffi::QMenu},
};

#[derive(Debug)]
pub struct SeparatorMenuItem {
    action: ActionHandle,
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let menu = parent.as_menu_container().as_qt::<QMenu>();
        let mut menu = RawPtr(menu);
        let action = unsafe { menu.pin_mut().addSeparator() }?;
        Ok(Self {
            action: ActionHandle::new(action),
        })
    }
}

impl AsMenuItem for SeparatorMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        unsafe { BorrowedMenuItem::qt(self.action.action_ptr()) }
    }
}
