use std::ptr::NonNull;

use gdk4_wayland::{WaylandDisplay, WaylandSurface, prelude::*, wayland_client::Proxy};
use gdk4_x11::{X11Display, X11Surface};
use gtk4::{
    glib::object::Cast,
    prelude::{NativeExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use wgpu::{
    CreateSurfaceError, Instance, Surface, SurfaceTarget,
    rwh::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
        RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
        XlibDisplayHandle, XlibWindowHandle,
    },
};
use winio_handle::{AsContainer, AsWidget};
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

    fn wayland(&self) -> Option<WindowHandleWrapper> {
        let native = self.as_widget().to_gtk().native()?;
        let surface = native
            .surface()?
            .dynamic_cast::<WaylandSurface>()
            .ok()?
            .wl_surface()?;
        let display = native
            .display()
            .dynamic_cast::<WaylandDisplay>()
            .ok()?
            .wl_display()?;
        let surface = RawWindowHandle::Wayland(WaylandWindowHandle::new(NonNull::new(
            surface.id().as_ptr().cast(),
        )?));
        let display = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(NonNull::new(
            display.id().as_ptr().cast(),
        )?));
        Some(WindowHandleWrapper(surface, display))
    }

    fn xcb(&self) -> Option<WindowHandleWrapper> {
        let native = self.as_widget().to_gtk().native()?;
        let window = native.surface()?.dynamic_cast::<X11Surface>().ok()?.xid();
        let display = native.display().dynamic_cast::<X11Display>().ok()?;
        let screen = display.screen().screen_number();
        let display = unsafe { display.xdisplay() };
        let handle = RawWindowHandle::Xlib(XlibWindowHandle::new(window));
        let display =
            RawDisplayHandle::Xlib(XlibDisplayHandle::new(NonNull::new(display.cast()), screen));
        Some(WindowHandleWrapper(handle, display))
    }

    pub fn create_surface(
        &self,
        instance: &Instance,
    ) -> std::result::Result<Surface<'static>, CreateSurfaceError> {
        let handle = self
            .wayland()
            .or_else(|| self.xcb())
            .ok_or_else(|| wgpu::wgc::instance::CreateSurfaceError::MissingDisplayHandle)?;

        let target = SurfaceTarget::from(handle);

        instance.create_surface(target)
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, widget);

#[derive(Debug)]
struct WindowHandleWrapper(RawWindowHandle, RawDisplayHandle);

unsafe impl Send for WindowHandleWrapper {}
unsafe impl Sync for WindowHandleWrapper {}

impl HasWindowHandle for WindowHandleWrapper {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        Ok(unsafe { WindowHandle::borrow_raw(self.0) })
    }
}

impl HasDisplayHandle for WindowHandleWrapper {
    fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(self.1) })
    }
}
