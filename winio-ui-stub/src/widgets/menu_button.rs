use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Image, Result, Widget, menu::MenuHandle, not_impl};

#[derive(Debug)]
pub struct MenuButton {
    widget: Widget,
    menu: MenuHandle,
}

#[inherit_methods(from = "self.widget")]
impl MenuButton {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        not_impl()
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn set_icon(&mut self, _icon: Option<&Image>) -> Result<()> {
        not_impl()
    }

    pub async fn start(&self) -> ! {
        not_impl()
    }
}

winio_handle::impl_as_widget!(MenuButton, widget);
winio_handle::impl_as_menu_container!(MenuButton, menu);
