use inherit_methods_macro::inherit_methods;
use wgpu::{CreateSurfaceError, Instance, Surface, SurfaceTargetUnsafe};
use windows::core::Interface;
use winio_handle::AsContainer;
use winio_primitive::{MouseButton, Point, Size, Vector};
use winui3::ISwapChainPanelNative;

use crate::{CanvasImpl, Result};

#[derive(Debug)]
pub struct WgpuCanvas {
    handle: CanvasImpl,
    native: ISwapChainPanelNative,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
        let native = handle.cast::<ISwapChainPanelNative>()?;
        Ok(Self { handle, native })
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

    pub fn create_surface(
        &self,
        instance: &Instance,
    ) -> std::result::Result<Surface<'static>, CreateSurfaceError> {
        unsafe {
            instance
                .create_surface_unsafe(SurfaceTargetUnsafe::SwapChainPanel(self.native.as_raw()))
        }
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);
