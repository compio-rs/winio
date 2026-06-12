use std::num::NonZero;

use inherit_methods_macro::inherit_methods;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle,
};
use wgpu::{CreateSurfaceError, Instance, Surface, SurfaceTarget};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{MouseButton, Point, Size, Vector};

use crate::{CanvasImpl, Result};

#[derive(Debug)]
pub struct WgpuCanvas {
    handle: CanvasImpl,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
        Ok(Self { handle })
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
        let handle = RawWindowHandle::Win32(Win32WindowHandle::new(
            NonZero::new(self.handle.as_widget().as_win32() as _).unwrap(),
        ));

        struct WindowHandleWrapper(RawWindowHandle);

        unsafe impl Send for WindowHandleWrapper {}
        unsafe impl Sync for WindowHandleWrapper {}

        impl HasWindowHandle for WindowHandleWrapper {
            fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
                Ok(unsafe { WindowHandle::borrow_raw(self.0) })
            }
        }

        impl HasDisplayHandle for WindowHandleWrapper {
            fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
                Ok(DisplayHandle::windows())
            }
        }

        let target = SurfaceTarget::from(WindowHandleWrapper(handle));

        instance.create_surface(target)
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);
