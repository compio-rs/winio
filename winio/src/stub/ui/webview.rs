use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct WebView {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(_parent: impl AsContainer) -> Result<Self> {
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

    pub fn source(&self) -> Result<String> {
        not_impl()
    }

    pub fn set_source(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn set_html(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        not_impl()
    }

    pub fn go_forward(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn can_go_back(&self) -> Result<bool> {
        not_impl()
    }

    pub fn go_back(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn reload(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn stop(&mut self) -> Result<()> {
        not_impl()
    }

    pub async fn wait_navigating(&self) {
        not_impl()
    }

    pub async fn wait_navigated(&self) {
        not_impl()
    }
}

winio_handle::impl_as_widget!(WebView, handle);
