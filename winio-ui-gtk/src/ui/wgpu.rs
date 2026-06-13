use inherit_methods_macro::inherit_methods;
use wgpu::{CreateSurfaceError, Instance, Surface};
use winio_handle::AsContainer;
use winio_primitive::{MouseButton, Point, Size, Vector};

use crate::{Canvas, Result};

#[derive(Debug)]
pub struct WgpuCanvas {
    widget: Canvas,
}

#[inherit_methods(from = "self.widget")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let canvas = Canvas::new(parent)?;
        Ok(Self { widget: canvas })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.widget.wait_mouse_down().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.widget.wait_mouse_up().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.widget.wait_mouse_move().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.widget.wait_mouse_wheel().await
    }

    pub fn create_surface(
        &self,
        _instance: &Instance,
    ) -> std::result::Result<Surface<'static>, CreateSurfaceError> {
        Err(wgpu::wgc::instance::CreateSurfaceError::MissingDisplayHandle.into())
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, widget);
