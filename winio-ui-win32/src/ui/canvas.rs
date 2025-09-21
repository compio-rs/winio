use std::ptr::null_mut;

use compio::driver::syscall;
use futures_util::FutureExt;
use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use windows::Win32::{
    Foundation::D2DERR_RECREATE_TARGET,
    Graphics::{
        Direct2D::{
            Common::{D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
            D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES,
            D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
            D2D1_RENDER_TARGET_TYPE_HARDWARE, D2D1_RENDER_TARGET_USAGE_NONE, ID2D1Factory,
            ID2D1HwndRenderTarget,
        },
        Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, POINT, SetLastError},
    Graphics::Gdi::MapWindowPoints,
    System::SystemServices::SS_OWNERDRAW,
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{
            GetParent, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL,
            WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WS_CHILD, WS_VISIBLE,
        },
    },
};
use winio_handle::{AsContainer, AsRawWindow};
use winio_primitive::{DrawingFont, MouseButton, Orient, Point, Rect, Size, Vector};
use winio_ui_windows_common::is_dark_mode_allowed_for_app;
pub use winio_ui_windows_common::{Brush, DrawingImage, DrawingPath, DrawingPathBuilder, Pen};

use crate::{
    RUNTIME,
    ui::{Widget, font::DWRITE_FACTORY},
};

#[inline]
fn d2d1<T>(f: impl FnOnce(&ID2D1Factory) -> T) -> T {
    RUNTIME.with(|runtime| f(runtime.d2d1()))
}

fn create_target(handle: HWND) -> ID2D1HwndRenderTarget {
    unsafe {
        d2d1(|d2d| {
            d2d.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_HARDWARE,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 0.0,
                    dpiY: 0.0,
                    usage: D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                },
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd: windows::Win32::Foundation::HWND(handle),
                    pixelSize: D2D_SIZE_U::default(),
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )
            .unwrap()
        })
    }
}

#[derive(Debug)]
pub struct Canvas {
    handle: Widget,
    target: ID2D1HwndRenderTarget,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
            0,
            parent.as_container().as_win32(),
        );
        let target = create_target(handle.as_raw_window().as_win32());
        Self { handle, target }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn context(&mut self) -> DrawingContext<'_> {
        unsafe {
            let size = self.handle.size_l2d(self.handle.size());
            loop {
                match self.target.Resize(&D2D_SIZE_U {
                    width: size.0 as u32,
                    height: size.1 as u32,
                }) {
                    Ok(()) => break,
                    Err(e) if e.code() == D2DERR_RECREATE_TARGET => self.handle_lost(),
                    Err(e) => panic!("{e:?}"),
                }
            }
            self.target.BeginDraw();
            self.target.Clear(Some(&if is_dark_mode_allowed_for_app() {
                D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }
            }));
        }
        DrawingContext::new(self)
    }

    fn handle_lost(&mut self) {
        self.target = create_target(self.handle.as_raw_window().as_win32());
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        loop {
            let (msg, b) = futures_util::select! {
                msg = self.handle.wait_parent(WM_LBUTTONDOWN).fuse() => (msg, MouseButton::Left),
                msg = self.handle.wait_parent(WM_RBUTTONDOWN).fuse() => (msg, MouseButton::Right),
                msg = self.handle.wait_parent(WM_MBUTTONDOWN).fuse() => (msg, MouseButton::Middle),
            };
            if self.is_in(msg.lparam(), false).is_some() {
                break b;
            }
        }
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        loop {
            let (msg, b) = futures_util::select! {
                msg = self.handle.wait_parent(WM_LBUTTONUP).fuse() => (msg, MouseButton::Left),
                msg = self.handle.wait_parent(WM_RBUTTONUP).fuse() => (msg, MouseButton::Right),
                msg = self.handle.wait_parent(WM_MBUTTONUP).fuse() => (msg, MouseButton::Middle),
            };
            if self.is_in(msg.lparam(), false).is_some() {
                break b;
            }
        }
    }

    pub async fn wait_mouse_move(&self) -> Point {
        loop {
            let msg = self.handle.wait_parent(WM_MOUSEMOVE).await;
            if let Some(p) = self.is_in(msg.lparam(), false) {
                break p;
            }
        }
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        let (msg, orient) = loop {
            let (msg, orient) = futures_util::select! {
                msg = self.handle.wait_parent(WM_MOUSEWHEEL).fuse() => (msg, Orient::Vertical),
                msg = self.handle.wait_parent(WM_MOUSEHWHEEL).fuse() => (msg, Orient::Horizontal),
            };
            if self.is_in(msg.lparam(), true).is_some() {
                break (msg, orient);
            }
        };
        let delta = ((msg.wparam() >> 16) & 0xFFFF) as i16 as isize;
        match orient {
            Orient::Vertical => Vector::new(0.0, delta as _),
            Orient::Horizontal => Vector::new(delta as _, 0.0),
        }
    }

    fn is_in(&self, lparam: LPARAM, screen: bool) -> Option<Point> {
        let (x, y) = ((lparam & 0xFFFF) as i32, ((lparam >> 16) & 0xFFFF) as i32);
        let mut p = POINT { x, y };
        let handle = self.handle.as_raw_window().as_win32();
        let parent = if screen {
            null_mut()
        } else {
            unsafe { GetParent(handle) }
        };
        unsafe { SetLastError(0) };
        match syscall!(BOOL, MapWindowPoints(parent, handle, &mut p, 1)) {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(0) => {}
            Err(e) => panic!("{e:?}"),
        }
        let p = self.handle.point_d2l((p.x, p.y));
        let size = self.size();
        if p.x >= 0.0 && p.x <= size.width && p.y >= 0.0 && p.y <= size.height {
            Some(p)
        } else {
            None
        }
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

pub struct DrawingContext<'a> {
    ctx: winio_ui_windows_common::DrawingContext,
    canvas: &'a mut Canvas,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        unsafe {
            match self.ctx.render_target().EndDraw(None, None) {
                Ok(()) => {}
                Err(e) if e.code() == D2DERR_RECREATE_TARGET => self.canvas.handle_lost(),
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}

impl<'a> DrawingContext<'a> {
    fn new(canvas: &'a mut Canvas) -> Self {
        Self {
            ctx: winio_ui_windows_common::DrawingContext::new(
                d2d1(|f| f.clone()),
                DWRITE_FACTORY.clone(),
                canvas.target.clone().into(),
            ),
            canvas,
        }
    }
}

#[inherit_methods(from = "self.ctx")]
impl DrawingContext<'_> {
    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath);

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath);

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64);

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64);

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64);

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect);

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect);

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point);

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect);

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect);

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size);

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size);

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str);

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage;

    pub fn draw_image(&mut self, image: &DrawingImage, rect: Rect, clip: Option<Rect>);

    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder;
}
