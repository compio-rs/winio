use compio_log::error;
use inherit_methods_macro::inherit_methods;
use wgpu::{Adapter, Instance, SurfaceTargetUnsafe};
use windows_core::Interface;
use winio_handle::AsContainer;
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};
use winio_wgpu::SurfaceCanvas;
use winui3::Microsoft::UI::Xaml::Media::DxInterop::ISwapChainPanelNative;

use crate::{CanvasImpl, Result};

#[derive(Debug)]
pub struct WgpuCanvas {
    handle: CanvasImpl,
    native: ISwapChainPanelNative,
    surface: SurfaceCanvas,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
        let native = handle.cast::<ISwapChainPanelNative>()?;
        Ok(Self {
            handle,
            native,
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
    ) -> Option<(wgpu::Texture, wgpu::TextureView, wgpu::TextureFormat)> {
        let size = self.handle.size().ok()?;
        let width = size.width as u32;
        let height = size.height as u32;
        let native = self.native.clone();
        self.surface
            .acquire_with(device, (width, height), move |instance| unsafe {
                instance
                    .create_surface_unsafe(SurfaceTargetUnsafe::SwapChainPanel(native.as_raw()))
                    .ok()
            })
    }

    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter) {
        self.surface.bind(instance, adapter);
    }

    pub fn present(&mut self, queue: &wgpu::Queue) -> Result<()> {
        self.surface.present(queue);
        Ok(())
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);

impl Drop for WgpuCanvas {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = self.native.SetSwapChain(None) {
                error!("Failed to clear swap chain: {e:?}")
            }
        }
    }
}
