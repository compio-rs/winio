use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use compio_log::error;
use inherit_methods_macro::inherit_methods;
use wgpu::{Texture, TextureFormat, TextureView};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};

use crate::{
    GlobalRuntime, QtMouseButton, Result, Widget,
    common::QString,
    platform::{KeyCharCallback, key_code},
};

const FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;
const FORMAT_SRGB: TextureFormat = TextureFormat::Bgra8UnormSrgb;

/// The offscreen texture rendered by `wgpu` and the staging buffer used to
/// copy it back.
#[derive(Debug)]
struct Frame {
    device: wgpu::Device,
    texture: Texture,
    view: TextureView,
    buffer: wgpu::Buffer,
    size: (u32, u32),
    padded_bytes_per_row: u32,
}

impl Frame {
    fn new(device: &wgpu::Device, (width, height): (u32, u32)) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("winio wgpu canvas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[FORMAT_SRGB],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(FORMAT_SRGB),
            ..Default::default()
        });
        let bytes_per_row = width * 4;
        let aligned = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = bytes_per_row.div_ceil(aligned) * aligned;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("winio wgpu canvas read-back"),
            size: padded_bytes_per_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            device: device.clone(),
            texture,
            view,
            buffer,
            size: (width, height),
            padded_bytes_per_row,
        }
    }

    fn present(&self, queue: &wgpu::Queue) -> Option<Vec<u8>> {
        let (width, height) = self.size;
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);

        let slice = self.buffer.slice(..);
        let mapped = Arc::new(AtomicBool::new(false));
        let mapped_cb = mapped.clone();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            mapped_cb.store(result.is_ok(), Ordering::Release);
        });
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        if !mapped.load(Ordering::Acquire) {
            return None;
        }

        let view = slice.get_mapped_range().ok()?;
        let bytes_per_row = (width * 4) as usize;
        let mut pixels = Vec::with_capacity(bytes_per_row * height as usize);
        for row in 0..height as usize {
            let start = row * self.padded_bytes_per_row as usize;
            pixels.extend_from_slice(&view[start..start + bytes_per_row]);
        }
        drop(view);
        self.buffer.unmap();
        Some(pixels)
    }
}

#[derive(Debug)]
pub struct WgpuCanvas {
    on_move: Box<Callback<Point>>,
    on_press: Box<Callback<MouseButton>>,
    on_release: Box<Callback<MouseButton>>,
    on_wheel: Box<Callback<Vector>>,
    on_key_down: Box<Callback<KeyCode>>,
    on_key_up: Box<Callback<KeyCode>>,
    on_key_char: Box<KeyCharCallback>,
    frame: RefCell<Option<Frame>>,
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
            frame: RefCell::new(None),
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
        &self,
        device: &wgpu::Device,
    ) -> Option<(Texture, TextureView, TextureFormat, (u32, u32))> {
        let size = self.widget.size().ok()?;
        let ratio = ffi::wgpu_canvas_pixel_ratio(self.widget.as_ref());
        let width = ((size.width * ratio).round() as u32).max(1);
        let height = ((size.height * ratio).round() as u32).max(1);
        let wanted = (width, height);

        let mut frame = self.frame.borrow_mut();
        if frame.as_ref().map(|f| f.size) != Some(wanted) {
            *frame = Some(Frame::new(device, wanted));
        }
        let frame = frame.as_ref()?;
        Some((
            frame.texture.clone(),
            frame.view.clone(),
            FORMAT_SRGB,
            frame.size,
        ))
    }

    pub fn present(&self, queue: &wgpu::Queue) -> Result<()> {
        let frame = self.frame.borrow();
        let Some(frame) = frame.as_ref() else {
            return Ok(());
        };
        match frame.present(queue) {
            Some(pixels) => unsafe {
                ffi::wgpu_canvas_set_image(
                    self.widget.as_ref(),
                    &pixels,
                    frame.size.0,
                    frame.size.1,
                    frame.size.0 * 4,
                );
            },
            None => error!("WGPU canvas: cannot read back the frame"),
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
