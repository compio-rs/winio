use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuContainer, BorrowedMenuItem};

use crate::{Image, Result, not_impl};

mod bar;
pub use bar::*;

mod bar_item;
pub use bar_item::*;

mod item;
pub use item::*;

mod popup;
pub use popup::*;

mod check;
pub use check::*;

mod radio;
pub use radio::*;

mod separator;
pub use separator::*;

#[derive(Debug)]
pub(crate) struct MenuHandle;

impl MenuHandle {
    pub fn is_visible(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_visible(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_enabled(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn text(&self) -> Result<String> {
        not_impl()
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn set_icon(&mut self, _icon: Option<&Image>) -> Result<()> {
        not_impl()
    }

    pub fn is_checked(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_checked(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }
}

impl AsMenuContainer for MenuHandle {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        not_impl()
    }
}

impl AsMenuItem for MenuHandle {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        not_impl()
    }
}
