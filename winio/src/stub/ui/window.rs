use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct Window {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Window {
    pub fn new() -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn client_size(&self) -> Result<Size> {
        not_impl()
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_size(&self) {
        not_impl()
    }

    pub async fn wait_move(&self) {
        not_impl()
    }

    pub async fn wait_close(&self) {
        not_impl()
    }

    pub async fn wait_theme_changed(&self) {
        not_impl()
    }
}

winio_handle::impl_as_window!(Window, handle);
winio_handle::impl_as_container!(Window, handle);

#[derive(Debug)]
pub struct View {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;
}

winio_handle::impl_as_container!(View, handle);
winio_handle::impl_as_widget!(View, handle);
