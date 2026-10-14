use std::fmt::Debug;

use winio_handle::{AsMenuBar, AsWindow, BorrowedMenuBar};

use crate::{
    Error, Result,
    menu::{
        RawPtr,
        ffi::{QMenuBar, menu_bar_exists, new_menu_bar},
    },
    widgets::QWidget,
};

#[derive(Debug)]
pub struct MenuBar {
    menu_bar: RawPtr<QMenuBar>,
}

impl MenuBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let window = parent.as_window().as_qt::<QWidget>();
        if unsafe { menu_bar_exists(window) } {
            return Err(Error::MenuBarExists);
        }
        let menu_bar = unsafe { new_menu_bar(window) }?;
        Ok(Self {
            menu_bar: RawPtr(menu_bar),
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(unsafe { self.menu_bar.as_ref() }.isVisible()?)
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        unsafe { self.menu_bar.pin_mut() }.setVisible(visible)?;
        Ok(())
    }
}

impl AsMenuBar for MenuBar {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        unsafe { BorrowedMenuBar::qt(self.menu_bar.0) }
    }
}
