use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
#[cfg(windows)]
pub use winio_ui_windows_common::Backdrop;

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

    #[cfg(windows)]
    pub fn set_icon_by_id(&mut self, _id: u16) -> Result<()> {
        not_impl()
    }

    #[cfg(windows)]
    pub fn backdrop(&self) -> Result<Backdrop> {
        not_impl()
    }

    #[cfg(windows)]
    pub fn set_backdrop(&mut self, _backdrop: Backdrop) -> Result<()> {
        not_impl()
    }

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
