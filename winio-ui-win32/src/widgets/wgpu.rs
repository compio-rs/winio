use std::num::NonZero;

use inherit_methods_macro::inherit_methods;
use wgpu::{
    Adapter, Instance, Queue, Texture, TextureFormat, TextureView,
    rwh::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle},
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};
use winio_wgpu::SurfaceCanvas;

use crate::{
    CanvasImpl, Result,
    platform::dpi::{DpiAware, get_dpi_for_window},
};

#[derive(Debug)]
pub struct WgpuCanvas {
    handle: CanvasImpl,
    surface: SurfaceCanvas,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
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

    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter) {
        self.surface.bind(instance, adapter);
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<(Texture, TextureView, TextureFormat)> {
        let hwnd = self.handle.as_widget().as_win32();
        let dpi = get_dpi_for_window(hwnd);
        let size = self.handle.size().ok()?;
        let width = size.width.to_device(dpi).max(1.0) as u32;
        let height = size.height.to_device(dpi).max(1.0) as u32;
        let window = RawWindowHandle::Win32(Win32WindowHandle::new(NonZero::new(hwnd as _)?));
        let display = RawDisplayHandle::Windows(WindowsDisplayHandle::new());
        self.surface
            .acquire(device, window, display, (width, height))
    }

    pub fn present(&mut self, queue: &Queue) -> Result<()> {
        self.surface.present(queue);
        Ok(())
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);
