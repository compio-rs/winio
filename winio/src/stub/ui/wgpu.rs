use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{MouseButton, Point, Size, Vector};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct WgpuCanvas {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_mouse_move(&self) -> Point {
        not_impl()
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        not_impl()
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        not_impl()
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        not_impl()
    }

    pub fn create_surface(
        &self,
        _instance: &wgpu::Instance,
    ) -> std::result::Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
        not_impl()
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);
