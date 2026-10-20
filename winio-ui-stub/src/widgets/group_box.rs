use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct GroupBox {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn client_size(&self) -> Result<Size> {
        not_impl()
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_widget!(GroupBox, handle);
winio_handle::impl_as_container!(GroupBox, handle);
