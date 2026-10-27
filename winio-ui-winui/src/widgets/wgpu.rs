use compio_log::error;
use inherit_methods_macro::inherit_methods;
use wgpu::{Adapter, Instance, SurfaceTargetUnsafe};
use windows::Win32::Graphics::Dxgi::DXGI_MATRIX_3X2_F;
use windows_core::{HRESULT, Interface};
use winio_handle::AsContainer;
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};
use winio_wgpu::SurfaceCanvas;
use winui3::Microsoft::UI::Xaml::Media::DxInterop::ISwapChainPanelNative;

use crate::{CanvasImpl, Error, Result};

#[derive(Debug)]
pub struct WgpuCanvas {
    surface: SurfaceCanvas,
    native: ISwapChainPanelNative,
    handle: CanvasImpl,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
        let native = handle.cast::<ISwapChainPanelNative>()?;
        Ok(Self {
            surface: SurfaceCanvas::new(),
            native,
            handle,
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
        let (scalex, scaley) = self.handle.composition_scale();
        let size = self.handle.size().ok()?;
        let width = (size.width as f32 * scalex).max(1.0) as u32;
        let height = (size.height as f32 * scaley).max(1.0) as u32;
        let native = self.native.clone();
        // SAFETY: `surface` is declared before `native` and `handle`, so it is
        // dropped before the panel that owns the native handle.
        let frame = unsafe {
            self.surface
                .acquire_with(device, (width, height), move |instance| {
                    instance
                        .create_surface_unsafe(SurfaceTargetUnsafe::SwapChainPanel(native.as_raw()))
                        .ok()
                })
        }?;
        if let Err(e) = self.apply_composition_scale(scalex, scaley) {
            error!("Failed to apply composition scale: {e:?}");
        }
        Some(frame)
    }

    fn apply_composition_scale(&self, scalex: f32, scaley: f32) -> Result<()> {
        if scalex <= 0.0 || scaley <= 0.0 {
            return Ok(());
        }
        let Some(surface) = self.surface.surface() else {
            return Ok(());
        };
        let hal = unsafe { surface.as_hal::<wgpu::hal::dx12::Api>() };
        let Some(hal) = hal else { return Ok(()) };
        let Some(swap_chain) = hal.swap_chain() else {
            return Ok(());
        };
        let matrix = DXGI_MATRIX_3X2_F {
            _11: 1.0 / scalex,
            _12: 0.0,
            _21: 0.0,
            _22: 1.0 / scaley,
            _31: 0.0,
            _32: 0.0,
        };
        // Convert across versions of `windows` crate
        unsafe { swap_chain.SetMatrixTransform(&raw const matrix as _) }
            .map_err(|e| Error::from_hresult(HRESULT(e.code().0)))?;
        Ok(())
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
