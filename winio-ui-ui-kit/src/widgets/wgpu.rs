use std::ptr::NonNull;

use inherit_methods_macro::inherit_methods;
use objc2::rc::Retained;
use wgpu::{
    Adapter, Instance, Queue, Texture, TextureFormat, TextureView,
    rwh::{RawDisplayHandle, RawWindowHandle, UiKitDisplayHandle, UiKitWindowHandle},
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};
use winio_wgpu::SurfaceCanvas;

use crate::{CanvasImpl, Result};

#[derive(Debug)]
pub struct WgpuCanvas {
    handle: CanvasImpl,
    surface: SurfaceCanvas,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent, false)?;
        Ok(Self {
            handle,
            surface: SurfaceCanvas::new(),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.handle.wait_mouse_down().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.handle.wait_mouse_up().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.handle.wait_mouse_move().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.handle.wait_mouse_wheel().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.handle.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.handle.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.handle.wait_key_char().await
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<(Texture, TextureView, TextureFormat, (u32, u32))> {
        let size = self.handle.size().ok()?;
        let scale = self.handle.scale_factor();
        let width = (size.width * scale).max(1.0) as u32;
        let height = (size.height * scale).max(1.0) as u32;
        let window = RawWindowHandle::UiKit(UiKitWindowHandle::new(NonNull::new(
            Retained::as_ptr(self.handle.as_widget().as_ui_kit())
                .cast_mut()
                .cast(),
        )?));
        let display = RawDisplayHandle::UiKit(UiKitDisplayHandle::new());
        self.surface
            .acquire(device, window, display, (width, height))
    }

    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter) {
        self.surface.bind(instance, adapter);
    }

    pub fn present(&mut self, queue: &Queue) -> Result<()> {
        self.surface.present(queue);
        Ok(())
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);
