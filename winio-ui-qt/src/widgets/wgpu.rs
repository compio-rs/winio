use compio_log::error;
use inherit_methods_macro::inherit_methods;
use wgpu::{Texture, TextureFormat, TextureView};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};
use winio_wgpu::MemoryCanvas;

use crate::{
    GlobalRuntime, QtMouseButton, Result, Widget,
    common::QString,
    platform::{KeyCharCallback, key_code},
};

#[derive(Debug)]
pub struct WgpuCanvas {
    on_move: Box<Callback<Point>>,
    on_press: Box<Callback<MouseButton>>,
    on_release: Box<Callback<MouseButton>>,
    on_wheel: Box<Callback<Vector>>,
    on_key_down: Box<Callback<KeyCode>>,
    on_key_up: Box<Callback<KeyCode>>,
    on_key_char: Box<KeyCharCallback>,
    frame: MemoryCanvas,
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
        let on_key_down = Box::new(Callback::new());
        let on_key_up = Box::new(Callback::new());
        let on_key_char = Box::<KeyCharCallback>::default();
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
            ffi::wgpu_canvas_register_key_down_event(
                widget.pin_mut(),
                Self::on_key_down,
                on_key_down.as_ref() as *const _ as _,
            )?;
            ffi::wgpu_canvas_register_key_up_event(
                widget.pin_mut(),
                Self::on_key_up,
                on_key_up.as_ref() as *const _ as _,
            )?;
            ffi::wgpu_canvas_register_key_char_event(
                widget.pin_mut(),
                Self::on_key_char,
                on_key_char.as_ref() as *const _ as _,
            )?;
        }
        Ok(Self {
            on_move,
            on_press,
            on_release,
            on_wheel,
            on_key_down,
            on_key_up,
            on_key_char,
            frame: MemoryCanvas::new(),
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

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<(Texture, TextureView, TextureFormat, (u32, u32))> {
        let size = self.widget.size().ok()?;
        let ratio = ffi::wgpu_canvas_pixel_ratio(self.widget.as_ref());
        let width = ((size.width * ratio).round() as u32).max(1);
        let height = ((size.height * ratio).round() as u32).max(1);
        Some(self.frame.acquire(device, (width, height)))
    }

    pub fn present(&mut self, queue: &wgpu::Queue) -> Result<()> {
        let Some(size) = self.frame.size() else {
            return Ok(());
        };
        let Some(pixels) = self.frame.present(queue) else {
            error!("WGPU canvas: cannot read back the frame");
            return Ok(());
        };
        unsafe {
            ffi::wgpu_canvas_set_image(self.widget.as_ref(), pixels, size.0, size.1, size.0 * 4);
        }
        Ok(())
    }

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

    fn on_key_down(c: *const u8, key: i32) {
        let c = c as *const Callback<KeyCode>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(key_code(key));
        }
    }

    fn on_key_up(c: *const u8, key: i32) {
        let c = c as *const Callback<KeyCode>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(key_code(key));
        }
    }

    fn on_key_char(c: *const u8, text: &QString) {
        let c = c as *const KeyCharCallback;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(text)
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

    pub async fn wait_key_down(&self) -> KeyCode {
        self.on_key_down.wait().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.on_key_up.wait().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.on_key_char.wait().await
    }
}

impl WgpuCanvas {
    pub fn bind(&mut self, _instance: &wgpu::Instance, _adapter: &wgpu::Adapter) {}
}

winio_handle::impl_as_widget!(WgpuCanvas, widget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/wgpu.hpp");

        type QWidget = crate::QWidget;
        type QtMouseButton = crate::QtMouseButton;
        type QString = crate::common::QString;

        unsafe fn new_wgpu_canvas(parent: *mut QWidget) -> Result<UniquePtr<QWidget>>;
        fn wgpu_canvas_pixel_ratio(w: &QWidget) -> f64;
        unsafe fn wgpu_canvas_set_image(
            w: &QWidget,
            data: &[u8],
            width: u32,
            height: u32,
            stride: u32,
        );

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
        unsafe fn wgpu_canvas_register_key_down_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn wgpu_canvas_register_key_up_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn wgpu_canvas_register_key_char_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, &QString),
            data: *const u8,
        ) -> Result<()>;
    }
}
