use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Data, Result, Widget, not_impl, not_impl_fut};

#[derive(Debug)]
pub struct DropArea {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl DropArea {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        not_impl_fut().await
    }
}

winio_handle::impl_as_widget!(DropArea, handle);
winio_handle::impl_as_container!(DropArea, handle);
