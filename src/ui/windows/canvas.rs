use std::{io, mem::MaybeUninit, ptr::null, rc::Rc};

use compio::driver::syscall;
use futures_util::FutureExt;
use widestring::U16CString;
use windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::{
        Direct2D::{
            Common::{
                D2D_POINT_2F, D2D_RECT_F, D2D_SIZE_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED,
                D2D1_COLOR_F, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED,
                D2D1_FIGURE_END_OPEN, D2D1_PIXEL_FORMAT,
            },
            D2D1_ARC_SEGMENT, D2D1_ARC_SIZE_LARGE, D2D1_ARC_SIZE_SMALL, D2D1_BRUSH_PROPERTIES,
            D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
            D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES,
            D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
            D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
            D2D1_SWEEP_DIRECTION_CLOCKWISE, D2D1CreateFactory, ID2D1Brush, ID2D1Factory,
            ID2D1Geometry, ID2D1HwndRenderTarget, ID2D1RenderTarget,
        },
        DirectWrite::{
            DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_ITALIC,
            DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_NORMAL,
            DWriteCreateFactory, IDWriteFactory, IDWriteTextLayout,
        },
        Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
    core::Interface,
};
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Gdi::InvalidateRect,
    System::SystemServices::SS_OWNERDRAW,
    UI::{
        Controls::{DRAWITEMSTRUCT, WC_STATICW},
        WindowsAndMessaging::{
            WM_DRAWITEM, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
            WM_RBUTTONDOWN, WM_RBUTTONUP, WS_CHILD, WS_VISIBLE,
        },
    },
};

use super::darkmode::is_dark_mode_allowed_for_app;
use crate::{
    AsRawWindow, BrushPen, Color, DrawingFont, HAlign, MouseButton, Point, Rect, RectBox,
    RelativeToScreen, Rotation, Size, SolidColorBrush, VAlign, Widget,
};

#[derive(Debug)]
pub struct Canvas {
    handle: Widget,
    d2d: ID2D1Factory,
    dwrite: IDWriteFactory,
    target: ID2D1HwndRenderTarget,
}

impl Canvas {
    pub fn new(parent: impl AsRawWindow) -> io::Result<Rc<Self>> {
        let handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
            0,
            parent.as_raw_window(),
        )?;
        let d2d: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        let dwrite = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let target = unsafe {
            d2d.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
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
                    hwnd: windows::Win32::Foundation::HWND(handle.as_raw_window()),
                    pixelSize: D2D_SIZE_U::default(),
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )?
        };
        Ok(Rc::new(Self {
            handle,
            d2d,
            dwrite,
            target,
        }))
    }

    pub fn loc(&self) -> io::Result<Point> {
        self.handle.loc()
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> io::Result<Size> {
        self.handle.size()
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)
    }

    pub fn redraw(&self) -> io::Result<()> {
        syscall!(BOOL, unsafe {
            InvalidateRect(self.as_raw_window(), null(), 0)
        })?;
        Ok(())
    }

    pub async fn wait_redraw(&self) -> io::Result<DrawingContext> {
        loop {
            let msg = self.handle.wait_parent(WM_DRAWITEM).await;
            let ds = unsafe { &mut *(msg.lParam as *mut DRAWITEMSTRUCT) };
            if ds.hwndItem == self.as_raw_window() {
                break;
            }
        }
        self.context()
    }

    fn context(&self) -> io::Result<DrawingContext> {
        unsafe {
            let dpi = self.handle.dpi();
            let size = self.handle.size_l2d(self.handle.size()?);
            self.target.Resize(&D2D_SIZE_U {
                width: size.0 as u32,
                height: size.1 as u32,
            })?;
            self.target.BeginDraw();
            self.target
                .Clear(Some(&color_f(if is_dark_mode_allowed_for_app() {
                    Color::new(0, 0, 0, 255)
                } else {
                    Color::new(255, 255, 255, 255)
                })));
            self.target.SetDpi(dpi as f32, dpi as f32);
            let ctx = DrawingContext {
                target: self.target.clone().cast()?,
                d2d: self.d2d.clone(),
                dwrite: self.dwrite.clone(),
            };
            Ok(ctx)
        }
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        futures_util::select! {
            _ = self.handle.wait_parent(WM_LBUTTONDOWN).fuse() => MouseButton::Left,
            _ = self.handle.wait_parent(WM_RBUTTONDOWN).fuse() => MouseButton::Right,
            _ = self.handle.wait_parent(WM_MBUTTONDOWN).fuse() => MouseButton::Middle,
        }
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        futures_util::select! {
            _ = self.handle.wait_parent(WM_LBUTTONUP).fuse() => MouseButton::Left,
            _ = self.handle.wait_parent(WM_RBUTTONUP).fuse() => MouseButton::Right,
            _ = self.handle.wait_parent(WM_MBUTTONUP).fuse() => MouseButton::Middle,
        }
    }

    pub async fn wait_mouse_move(&self) -> io::Result<Point> {
        loop {
            let msg = self.handle.wait_parent(WM_MOUSEMOVE).await;
            let (x, y) = ((msg.lParam & 0xFFFF) as i32, (msg.lParam >> 16) as i32);
            let p = self.handle.point_d2l((x, y));
            let loc = self.loc()?;
            let size = self.size()?;
            if Rect::new(loc, size).contains(p) {
                break Ok((p - loc).to_point());
            }
        }
    }
}

impl AsRawWindow for Canvas {
    fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }
}

fn color_f(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: c.r as f32 / 255.0,
        g: c.g as f32 / 255.0,
        b: c.b as f32 / 255.0,
        a: c.a as f32 / 255.0,
    }
}

const fn point_2f(p: Point) -> D2D_POINT_2F {
    D2D_POINT_2F {
        x: p.x as f32,
        y: p.y as f32,
    }
}

const fn size_f(s: Size) -> D2D_SIZE_F {
    D2D_SIZE_F {
        width: s.width as f32,
        height: s.height as f32,
    }
}

fn rect_f(r: Rect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: r.origin.x as f32,
        top: r.origin.y as f32,
        right: (r.origin.x + r.size.width) as f32,
        bottom: (r.origin.y + r.size.height) as f32,
    }
}

pub struct DrawingContext {
    target: ID2D1RenderTarget,
    d2d: ID2D1Factory,
    dwrite: IDWriteFactory,
}

#[inline]
fn to_trans(rect: Rect) -> RelativeToScreen {
    RelativeToScreen::scale(rect.size.width, rect.size.height)
        .then_translate(rect.origin.to_vector())
}

fn get_arc(rect: Rect, start: f64, end: f64) -> (Size, Point, Point, Point) {
    let radius = rect.size / 2.0;
    let centerp = rect.origin.add_size(&radius);
    let startp = centerp + Rotation::radians(start).transform_vector(radius.to_vector());
    let endp = centerp + Rotation::radians(end).transform_vector(radius.to_vector());
    (radius, centerp, startp, endp)
}

fn ellipse(rect: Rect) -> D2D1_ELLIPSE {
    D2D1_ELLIPSE {
        point: point_2f(rect.origin.add_size(&(rect.size / 2.0))),
        radiusX: (rect.size.width / 2.0) as f32,
        radiusY: (rect.size.height / 2.0) as f32,
    }
}

impl DrawingContext {
    #[inline]
    fn get_brush(&self, brush: impl Brush, rect: Rect) -> io::Result<ID2D1Brush> {
        brush.create(&self.target, to_trans(rect))
    }

    #[inline]
    fn get_pen(&self, pen: impl Pen, rect: Rect) -> io::Result<(ID2D1Brush, f32)> {
        pen.create(&self.target, to_trans(rect))
    }

    fn get_arc_geo(
        &self,
        rect: Rect,
        start: f64,
        end: f64,
        close: bool,
    ) -> io::Result<ID2D1Geometry> {
        unsafe {
            let geo = self.d2d.CreatePathGeometry()?;
            let sink = geo.Open()?;
            let (radius, centerp, startp, endp) = get_arc(rect, start, end);
            sink.BeginFigure(point_2f(startp), D2D1_FIGURE_BEGIN_HOLLOW);
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: point_2f(endp),
                size: size_f(radius),
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: if (end - start) > std::f64::consts::PI {
                    D2D1_ARC_SIZE_LARGE
                } else {
                    D2D1_ARC_SIZE_SMALL
                },
            });
            if close {
                sink.AddLine(point_2f(centerp));
            }
            sink.EndFigure(if close {
                D2D1_FIGURE_END_CLOSED
            } else {
                D2D1_FIGURE_END_OPEN
            });
            sink.Close()?;
            Ok(geo.cast()?)
        }
    }

    fn get_str_layout(
        &self,
        font: DrawingFont,
        pos: Point,
        s: &str,
    ) -> io::Result<(Rect, IDWriteTextLayout)> {
        unsafe {
            let font_family = U16CString::from_str_truncate(font.family);
            let format = self.dwrite.CreateTextFormat(
                windows::core::PCWSTR::from_raw(font_family.as_ptr()),
                None,
                if font.bold {
                    DWRITE_FONT_WEIGHT_BOLD
                } else {
                    DWRITE_FONT_WEIGHT_NORMAL
                },
                if font.italic {
                    DWRITE_FONT_STYLE_ITALIC
                } else {
                    DWRITE_FONT_STYLE_NORMAL
                },
                DWRITE_FONT_STRETCH_NORMAL,
                font.size as f32,
                windows::core::w!(""),
            )?;
            let size = self.target.GetSize();
            let mut rect = Rect::new(pos, pos.to_vector().to_size());
            let s = U16CString::from_str_truncate(s);
            let layout = self.dwrite.CreateTextLayout(
                s.as_slice_with_nul(),
                &format,
                size.width,
                size.height,
            )?;
            let mut metrics = MaybeUninit::uninit();
            layout.GetMetrics(metrics.as_mut_ptr())?;
            let metrics = metrics.assume_init();
            match font.halign {
                HAlign::Center => {
                    rect.origin.x -= metrics.width as f64 / 2.0;
                }
                HAlign::Right => {
                    rect.origin.x -= metrics.width as f64;
                }
                _ => {}
            }
            match font.valign {
                VAlign::Center => {
                    rect.origin.y -= metrics.height as f64 / 2.0;
                }
                VAlign::Bottom => {
                    rect.origin.y -= metrics.height as f64;
                }
                _ => {}
            }
            rect.size = Size::new(metrics.width as f64, metrics.height as f64);
            Ok((rect, layout))
        }
    }

    pub fn draw_arc(&self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> io::Result<()> {
        let geo = self.get_arc_geo(rect, start, end, false)?;
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawGeometry(&geo, &b, width, None);
        }
        Ok(())
    }

    pub fn fill_pie(&self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> io::Result<()> {
        let geo = self.get_arc_geo(rect, start, end, true)?;
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillGeometry(&geo, &b, None);
        }
        Ok(())
    }

    pub fn draw_ellipse(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        let e = ellipse(rect);
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawEllipse(&e, &b, width, None);
        }
        Ok(())
    }

    pub fn fill_ellipse(&self, brush: impl Brush, rect: Rect) -> io::Result<()> {
        let e = ellipse(rect);
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillEllipse(&e, &b);
        }
        Ok(())
    }

    pub fn draw_line(&self, pen: impl Pen, start: Point, end: Point) -> io::Result<()> {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target
                .DrawLine(point_2f(start), point_2f(end), &b, width, None);
        }
        Ok(())
    }

    pub fn draw_rect(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawRectangle(&rect_f(rect), &b, width, None);
        }
        Ok(())
    }

    pub fn fill_rect(&self, brush: impl Brush, rect: Rect) -> io::Result<()> {
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillRectangle(&rect_f(rect), &b);
        }
        Ok(())
    }

    pub fn draw_round_rect(&self, pen: impl Pen, rect: Rect, round: Size) -> io::Result<()> {
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: rect_f(rect),
                    radiusX: round.width as f32,
                    radiusY: round.height as f32,
                },
                &b,
                width,
                None,
            );
        }
        Ok(())
    }

    pub fn fill_round_rect(&self, brush: impl Brush, rect: Rect, round: Size) -> io::Result<()> {
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: rect_f(rect),
                    radiusX: round.width as f32,
                    radiusY: round.height as f32,
                },
                &b,
            );
        }
        Ok(())
    }

    pub fn draw_str(
        &self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) -> io::Result<()> {
        let (rect, layout) = self.get_str_layout(font, pos, text.as_ref())?;
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.DrawTextLayout(
                point_2f(rect.origin),
                &layout,
                &b,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
        }
        Ok(())
    }
}

impl Drop for DrawingContext {
    fn drop(&mut self) {
        unsafe { self.target.EndDraw(None, None) }.unwrap();
    }
}

const MATRIX_IDENTITY: Matrix3x2 = Matrix3x2 {
    M11: 1.0,
    M12: 0.0,
    M21: 0.0,
    M22: 1.0,
    M31: 0.0,
    M32: 0.0,
};

const BRUSH_PROPERTIES_DEFAULT: D2D1_BRUSH_PROPERTIES = D2D1_BRUSH_PROPERTIES {
    opacity: 1.0,
    transform: MATRIX_IDENTITY,
};

pub trait Brush {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToScreen)
    -> io::Result<ID2D1Brush>;
}

impl<B: Brush> Brush for &'_ B {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToScreen,
    ) -> io::Result<ID2D1Brush> {
        (**self).create(target, trans)
    }
}

impl Brush for SolidColorBrush {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        _trans: RelativeToScreen,
    ) -> io::Result<ID2D1Brush> {
        Ok(unsafe {
            target.CreateSolidColorBrush(&color_f(self.color), Some(&BRUSH_PROPERTIES_DEFAULT))?
        }
        .cast()?)
    }
}

pub trait Pen {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToScreen,
    ) -> io::Result<(ID2D1Brush, f32)>;
}

impl<P: Pen> Pen for &'_ P {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToScreen,
    ) -> io::Result<(ID2D1Brush, f32)> {
        (**self).create(target, trans)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToScreen,
    ) -> io::Result<(ID2D1Brush, f32)> {
        let brush = self.brush.create(target, trans)?;
        Ok((brush, self.width as _))
    }
}
