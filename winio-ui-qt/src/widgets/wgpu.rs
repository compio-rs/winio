use std::{num::NonZero, ptr::NonNull};

use inherit_methods_macro::inherit_methods;
use wgpu::{
    CreateSurfaceError, Instance, Surface, SurfaceTarget,
    rwh::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
        RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle, XcbDisplayHandle,
        XcbWindowHandle,
    },
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{MouseButton, Point, Size, Vector};

use crate::{GlobalRuntime, QtMouseButton, Result, Widget};

#[derive(Debug)]
pub struct WgpuCanvas {
    on_move: Box<Callback<Point>>,
    on_press: Box<Callback<MouseButton>>,
    on_release: Box<Callback<MouseButton>>,
    on_wheel: Box<Callback<Vector>>,
    widget: Widget<ffi::QWidget>,
}

#[inherit_methods(from = "self.widget")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_wgpu_canvas(parent.as_container().as_qt()) }?;
        widget.pin_mut().setVisible(true)?;
        let on_move = Box::new(Callback::new());
        let on_press = Box::new(Callback::new());
        let on_release = Box::new(Callback::new());
        let on_wheel = Box::new(Callback::new());
        unsafe {
            ffi::wgpu_canvas_register_move_event(
                widget.pin_mut(),
                Self::on_move,
                on_move.as_ref() as *const _ as _,
            )?;
            ffi::wgpu_canvas_register_press_event(
                widget.pin_mut(),
                Self::on_press,
                on_press.as_ref() as *const _ as _,
            )?;
            ffi::wgpu_canvas_register_release_event(
                widget.pin_mut(),
                Self::on_release,
                on_release.as_ref() as *const _ as _,
            )?;
            ffi::wgpu_canvas_register_wheel_event(
                widget.pin_mut(),
                Self::on_wheel,
                on_wheel.as_ref() as *const _ as _,
            )?;
        }
        Ok(Self {
            on_move,
            on_press,
            on_release,
            on_wheel,
            widget: Widget::new(widget)?,
        })
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

    fn on_move(c: *const u8, x: i32, y: i32) {
        let c = c as *const Callback<Point>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(Point::new(x as _, y as _));
        }
    }

    fn on_press(c: *const u8, m: QtMouseButton) {
        let c = c as *const Callback<MouseButton>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(m.into());
        }
    }

    fn on_release(c: *const u8, m: QtMouseButton) {
        let c = c as *const Callback<MouseButton>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(m.into());
        }
    }

    fn on_wheel(c: *const u8, x: i32, y: i32) {
        let c = c as *const Callback<Vector>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(Vector::new(x as _, y as _));
        }
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.on_press.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.on_release.wait().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.on_move.wait().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.on_wheel.wait().await
    }

    fn wayland(&self) -> Option<WindowHandleWrapper> {
        let desc = ffi::wgpu_canvas_wayland_descriptor(self.widget.as_ref()).ok()?;
        let display = NonNull::new(desc.display)?;
        let surface = NonNull::new(desc.surface)?;
        let handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface.cast()));
        let display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display.cast()));
        Some(WindowHandleWrapper(handle, display_handle))
    }

    fn xcb(&self) -> Option<WindowHandleWrapper> {
        let desc = ffi::wgpu_canvas_xcb_descriptor(self.widget.as_ref()).ok()?;
        let window = NonZero::new(desc.window)?;
        let handle = RawWindowHandle::Xcb(XcbWindowHandle::new(window));
        let display_handle = RawDisplayHandle::Xcb(XcbDisplayHandle::new(
            NonNull::new(desc.connection).map(|p| p.cast()),
            desc.screen,
        ));
        Some(WindowHandleWrapper(handle, display_handle))
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

#[cxx::bridge]
mod ffi {
    struct WaylandDescriptor {
        display: *mut wl_display,
        surface: *mut wl_surface,
    }

    struct XcbDescriptor {
        connection: *mut xcb_connection_t,
        screen: i32,
        window: u32,
    }

    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/wgpu.hpp");

        type QWidget = crate::QWidget;
        type QtMouseButton = crate::QtMouseButton;
        type wl_display;
        type wl_surface;
        type xcb_connection_t;

        unsafe fn new_wgpu_canvas(parent: *mut QWidget) -> Result<UniquePtr<QWidget>>;
        unsafe fn wgpu_canvas_register_move_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn wgpu_canvas_register_press_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, QtMouseButton),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn wgpu_canvas_register_release_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, QtMouseButton),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn wgpu_canvas_register_wheel_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        ) -> Result<()>;

        fn wgpu_canvas_wayland_descriptor(w: &QWidget) -> Result<WaylandDescriptor>;
        fn wgpu_canvas_xcb_descriptor(w: &QWidget) -> Result<XcbDescriptor>;
    }
}
