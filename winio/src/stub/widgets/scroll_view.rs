use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct ScrollView {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn hscroll(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_hscroll(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn vscroll(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_vscroll(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub async fn start(&self) -> ! {
        not_impl()
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);
winio_handle::impl_as_container!(ScrollView, handle);
