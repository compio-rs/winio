use std::{marker::PhantomData, mem::MaybeUninit};

use futures_util::FutureExt;
use image::{DynamicImage, Pixel, Rgba};
use widestring::U16CString;
use windows::{
    Win32::Graphics::{
        Direct2D::{
            Common::{
                D2D_RECT_F, D2D_SIZE_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED,
                D2D1_BEZIER_SEGMENT, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_HOLLOW,
                D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D1_GRADIENT_STOP,
                D2D1_PIXEL_FORMAT,
            },
            D2D1_ARC_SEGMENT, D2D1_ARC_SIZE_LARGE, D2D1_ARC_SIZE_SMALL,
            D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR, D2D1_BITMAP_PROPERTIES,
            D2D1_BRUSH_PROPERTIES, D2D1_DEFAULT_FLATTENING_TOLERANCE, D2D1_DRAW_TEXT_OPTIONS_NONE,
            D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_FACTORY_TYPE_SINGLE_THREADED,
            D2D1_FEATURE_LEVEL_DEFAULT, D2D1_GAMMA_2_2, D2D1_HWND_RENDER_TARGET_PROPERTIES,
            D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE,
            D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES,
            D2D1_RENDER_TARGET_TYPE_HARDWARE, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
            D2D1_SWEEP_DIRECTION_CLOCKWISE, D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
            D2D1CreateFactory, ID2D1Bitmap, ID2D1Brush, ID2D1Factory, ID2D1Geometry,
            ID2D1GeometrySink, ID2D1HwndRenderTarget, ID2D1PathGeometry, ID2D1RenderTarget,
        },
        DirectWrite::{
            DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_NORMAL, IDWriteTextLayout,
        },
        Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM},
    },
    core::Interface,
};
use windows_numerics::{Matrix3x2, Vector2};
use windows_sys::Win32::{
    Foundation::LPARAM,
    System::SystemServices::SS_OWNERDRAW,
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{
            WM_DRAWITEM, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
            WM_RBUTTONDOWN, WM_RBUTTONUP, WS_CHILD, WS_VISIBLE,
        },
    },
};

use crate::{
    AsRawWindow, AsWindow, BrushPen, Color, DrawingFont, GradientStop, HAlign, LinearGradientBrush,
    MouseButton, Point, RadialGradientBrush, Rect, RectBox, RelativeToLogical, Size,
    SolidColorBrush, VAlign, Vector,
    runtime::WindowMessageDetail,
    ui::{Widget, darkmode::is_dark_mode_allowed_for_app, font::DWRITE_FACTORY},
};

#[derive(Debug)]
pub struct Canvas {
    handle: Widget,
    d2d: ID2D1Factory,
    target: ID2D1HwndRenderTarget,
}

impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
            0,
            parent.as_window().as_raw_window(),
        );
        let d2d: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let target = unsafe {
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
                    hwnd: windows::Win32::Foundation::HWND(handle.as_raw_window()),
                    pixelSize: D2D_SIZE_U::default(),
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )
            .unwrap()
        };
        Self {
            handle,
            d2d,
            target,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn context(&mut self) -> DrawingContext<'_> {
        unsafe {
            let size = self.handle.size_l2d(self.handle.size());
            self.target
                .Resize(&D2D_SIZE_U {
                    width: size.0 as u32,
                    height: size.1 as u32,
                })
                .unwrap();
            self.target.BeginDraw();
            self.target
                .Clear(Some(&color_f(if is_dark_mode_allowed_for_app() {
                    Color::new(0, 0, 0, 255)
                } else {
                    Color::new(255, 255, 255, 255)
                })));
            DrawingContext {
                target: self.target.clone().cast().unwrap(),
                d2d: self.d2d.clone(),
                _p: PhantomData,
            }
        }
    }

    pub async fn wait_redraw(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_DRAWITEM).await;
            if let Some(WindowMessageDetail::DrawItem { handle }) = msg.detail {
                if std::ptr::eq(handle, self.handle.as_raw_window()) {
                    break;
                }
            }
        }
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        loop {
            let (msg, b) = futures_util::select! {
                msg = self.handle.wait_parent(WM_LBUTTONDOWN).fuse() => (msg, MouseButton::Left),
                msg = self.handle.wait_parent(WM_RBUTTONDOWN).fuse() => (msg, MouseButton::Right),
                msg = self.handle.wait_parent(WM_MBUTTONDOWN).fuse() => (msg, MouseButton::Middle),
            };
            if self.is_in(msg.lparam).is_some() {
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
            if self.is_in(msg.lparam).is_some() {
                break b;
            }
        }
    }

    pub async fn wait_mouse_move(&self) -> Point {
        loop {
            let msg = self.handle.wait_parent(WM_MOUSEMOVE).await;
            if let Some(p) = self.is_in(msg.lparam) {
                break p;
            }
        }
    }

    fn is_in(&self, lparam: LPARAM) -> Option<Point> {
        let (x, y) = ((lparam & 0xFFFF) as i32, (lparam >> 16) as i32);
        let p = self.handle.point_d2l((x, y));
        let loc = self.loc();
        let size = self.size();
        if Rect::new(loc, size).contains(p) {
            Some((p - loc).to_point())
        } else {
            None
        }
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

const fn point_2f(p: Point) -> Vector2 {
    Vector2 {
        X: p.x as f32,
        Y: p.y as f32,
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

pub fn gradient_stop(s: &GradientStop) -> D2D1_GRADIENT_STOP {
    D2D1_GRADIENT_STOP {
        position: s.pos as f32,
        color: color_f(s.color),
    }
}

pub struct DrawingContext<'a> {
    target: ID2D1RenderTarget,
    d2d: ID2D1Factory,
    _p: PhantomData<&'a Canvas>,
}

#[inline]
fn to_trans(rect: Rect) -> RelativeToLogical {
    RelativeToLogical::scale(rect.size.width, rect.size.height)
        .then_translate(rect.origin.to_vector())
}

fn get_arc(rect: Rect, start: f64, end: f64) -> (Size, Point, Point, Point) {
    let radius = rect.size / 2.0;
    let centerp = rect.origin.add_size(&radius);
    let startp = centerp + Vector::new(radius.width * start.cos(), radius.height * start.sin());
    let endp = centerp + Vector::new(radius.width * end.cos(), radius.height * end.sin());
    (radius, centerp, startp, endp)
}

fn ellipse(rect: Rect) -> D2D1_ELLIPSE {
    D2D1_ELLIPSE {
        point: point_2f(rect.origin.add_size(&(rect.size / 2.0))),
        radiusX: (rect.size.width / 2.0) as f32,
        radiusY: (rect.size.height / 2.0) as f32,
    }
}

impl DrawingContext<'_> {
    #[inline]
    fn get_brush(&self, brush: impl Brush, rect: Rect) -> ID2D1Brush {
        brush.create(&self.target, to_trans(rect))
    }

    #[inline]
    fn get_pen(&self, pen: impl Pen, rect: Rect) -> (ID2D1Brush, f32) {
        pen.create(&self.target, to_trans(rect))
    }

    fn get_arc_geo(&self, rect: Rect, start: f64, end: f64, close: bool) -> ID2D1Geometry {
        unsafe {
            let geo = self.d2d.CreatePathGeometry().unwrap();
            let sink = geo.Open().unwrap();
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
            sink.Close().unwrap();
            geo.cast().unwrap()
        }
    }

    fn get_str_layout(&self, font: DrawingFont, pos: Point, s: &str) -> (Rect, IDWriteTextLayout) {
        unsafe {
            let font_family = U16CString::from_str_truncate(font.family);
            let format = DWRITE_FACTORY
                .CreateTextFormat(
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
                )
                .unwrap();
            let size = self.target.GetSize();
            let mut rect = Rect::new(pos, pos.to_vector().to_size());
            let s = U16CString::from_str_truncate(s);
            let layout = DWRITE_FACTORY
                .CreateTextLayout(s.as_slice_with_nul(), &format, size.width, size.height)
                .unwrap();
            let mut metrics = MaybeUninit::uninit();
            layout.GetMetrics(metrics.as_mut_ptr()).unwrap();
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
            (rect, layout)
        }
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) {
        let width = pen.width();
        let rect = unsafe {
            path.geo
                .GetWidenedBounds(width, None, None, D2D1_DEFAULT_FLATTENING_TOLERANCE)
                .unwrap()
        };
        let (b, width) = self.get_pen(
            pen,
            RectBox::new(
                Point::new(rect.left as _, rect.top as _),
                Point::new(rect.right as _, rect.bottom as _),
            )
            .to_rect(),
        );
        unsafe {
            self.target.DrawGeometry(&path.geo, &b, width, None);
        }
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) {
        let rect = unsafe { path.geo.GetBounds(None).unwrap() };
        let b = self.get_brush(
            brush,
            RectBox::new(
                Point::new(rect.left as _, rect.top as _),
                Point::new(rect.right as _, rect.bottom as _),
            )
            .to_rect(),
        );
        unsafe {
            self.target.FillGeometry(&path.geo, &b, None);
        }
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let geo = self.get_arc_geo(rect, start, end, false);
        let (b, width) = self.get_pen(pen, rect);
        unsafe {
            self.target.DrawGeometry(&geo, &b, width, None);
        }
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let geo = self.get_arc_geo(rect, start, end, true);
        let (b, width) = self.get_pen(pen, rect);
        unsafe {
            self.target.DrawGeometry(&geo, &b, width, None);
        }
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        let geo = self.get_arc_geo(rect, start, end, true);
        let b = self.get_brush(brush, rect);
        unsafe {
            self.target.FillGeometry(&geo, &b, None);
        }
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        let e = ellipse(rect);
        let (b, width) = self.get_pen(pen, rect);
        unsafe {
            self.target.DrawEllipse(&e, &b, width, None);
        }
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        let e = ellipse(rect);
        let b = self.get_brush(brush, rect);
        unsafe {
            self.target.FillEllipse(&e, &b);
        }
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        let (b, width) = self.get_pen(pen, rect);
        unsafe {
            self.target
                .DrawLine(point_2f(start), point_2f(end), &b, width, None);
        }
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        let (b, width) = self.get_pen(pen, rect);
        unsafe {
            self.target.DrawRectangle(&rect_f(rect), &b, width, None);
        }
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        let b = self.get_brush(brush, rect);
        unsafe {
            self.target.FillRectangle(&rect_f(rect), &b);
        }
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        let (b, width) = self.get_pen(pen, rect);
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
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        let b = self.get_brush(brush, rect);
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
    }

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str) {
        let (rect, layout) = self.get_str_layout(font, pos, text.as_ref());
        let b = self.get_brush(brush, rect);
        unsafe {
            self.target.DrawTextLayout(
                point_2f(rect.origin),
                &layout,
                &b,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
        }
    }

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage {
        DrawingImage::new(&self.target, image)
    }

    pub fn draw_image(&mut self, image: &DrawingImage, rect: Rect, clip: Option<Rect>) {
        unsafe {
            let clip = clip.map(rect_f);
            self.target.DrawBitmap(
                &image.0,
                Some(&rect_f(rect)),
                1.0,
                D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                clip.as_ref().map(|r| r as *const _),
            );
        }
    }

    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder {
        DrawingPathBuilder::new(&self.d2d, start)
    }
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        unsafe { self.target.EndDraw(None, None) }.unwrap();
    }
}

pub struct DrawingPath {
    geo: ID2D1Geometry,
}

impl DrawingPath {
    fn new(geo: ID2D1Geometry) -> Self {
        Self { geo }
    }
}

pub struct DrawingPathBuilder {
    geo: ID2D1PathGeometry,
    sink: ID2D1GeometrySink,
}

impl DrawingPathBuilder {
    fn new(d2d: &ID2D1Factory, start: Point) -> Self {
        unsafe {
            let geo = d2d.CreatePathGeometry().unwrap();
            let sink = geo.Open().unwrap();
            sink.BeginFigure(point_2f(start), D2D1_FIGURE_BEGIN_HOLLOW);
            Self { geo, sink }
        }
    }

    pub fn add_line(&mut self, p: Point) {
        unsafe {
            self.sink.AddLine(point_2f(p));
        }
    }

    pub fn add_arc(&mut self, center: Point, radius: Size, start: f64, end: f64, clockwise: bool) {
        unsafe {
            let startp =
                center + Vector::new(radius.width * start.cos(), radius.height * start.sin());
            let endp = center + Vector::new(radius.width * end.cos(), radius.height * end.sin());
            self.add_line(startp);
            self.sink.AddArc(&D2D1_ARC_SEGMENT {
                point: point_2f(endp),
                size: size_f(radius),
                rotationAngle: 0.0,
                sweepDirection: if clockwise {
                    D2D1_SWEEP_DIRECTION_CLOCKWISE
                } else {
                    D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE
                },
                arcSize: if (end - start) > std::f64::consts::PI {
                    D2D1_ARC_SIZE_LARGE
                } else {
                    D2D1_ARC_SIZE_SMALL
                },
            });
        }
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) {
        unsafe {
            self.sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: point_2f(p1),
                point2: point_2f(p2),
                point3: point_2f(p3),
            });
        }
    }

    pub fn build(self, close: bool) -> DrawingPath {
        unsafe {
            self.sink.EndFigure(if close {
                D2D1_FIGURE_END_CLOSED
            } else {
                D2D1_FIGURE_END_OPEN
            });
            self.sink.Close().unwrap();
            DrawingPath::new(self.geo.cast().unwrap())
        }
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

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> ID2D1Brush;
}

impl<B: Brush> Brush for &'_ B {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> ID2D1Brush {
        (**self).create(target, trans)
    }
}

impl Brush for SolidColorBrush {
    fn create(&self, target: &ID2D1RenderTarget, _trans: RelativeToLogical) -> ID2D1Brush {
        unsafe {
            target
                .CreateSolidColorBrush(&color_f(self.color), Some(&BRUSH_PROPERTIES_DEFAULT))
                .unwrap()
                .cast()
                .unwrap()
        }
    }
}

impl Brush for LinearGradientBrush {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> ID2D1Brush {
        let props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
            startPoint: point_2f(trans.transform_point(self.start)),
            endPoint: point_2f(trans.transform_point(self.end)),
        };
        let stops = self.stops.iter().map(gradient_stop).collect::<Vec<_>>();
        unsafe {
            let stop_collection = target
                .CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
                .unwrap();
            target
                .CreateLinearGradientBrush(
                    &props,
                    Some(&BRUSH_PROPERTIES_DEFAULT),
                    &stop_collection,
                )
                .unwrap()
                .cast()
                .unwrap()
        }
    }
}

impl Brush for RadialGradientBrush {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> ID2D1Brush {
        let radius = self.radius.to_vector();
        let radius = trans.transform_vector(radius);
        let props = D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
            center: point_2f(trans.transform_point(self.center)),
            gradientOriginOffset: point_2f(
                trans.transform_vector(self.origin - self.center).to_point(),
            ),
            radiusX: radius.x as f32,
            radiusY: radius.y as f32,
        };
        let stops = self.stops.iter().map(gradient_stop).collect::<Vec<_>>();
        unsafe {
            let stop_collection = target
                .CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
                .unwrap();
            target
                .CreateRadialGradientBrush(
                    &props,
                    Some(&BRUSH_PROPERTIES_DEFAULT),
                    &stop_collection,
                )
                .unwrap()
                .cast()
                .unwrap()
        }
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> (ID2D1Brush, f32);
    #[doc(hidden)]
    fn width(&self) -> f32;
}

impl<P: Pen> Pen for &'_ P {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> (ID2D1Brush, f32) {
        (**self).create(target, trans)
    }

    fn width(&self) -> f32 {
        (**self).width()
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> (ID2D1Brush, f32) {
        let brush = self.brush.create(target, trans);
        (brush, self.width as _)
    }

    fn width(&self) -> f32 {
        self.width as _
    }
}

pub struct DrawingImage(ID2D1Bitmap);

impl DrawingImage {
    fn new(target: &ID2D1RenderTarget, image: DynamicImage) -> Self {
        let image = match image {
            DynamicImage::ImageRgba8(image) => image,
            _ => image.into_rgba8(),
        };
        let mut dpix = 0.0;
        let mut dpiy = 0.0;
        unsafe { target.GetDpi(&mut dpix, &mut dpiy) };
        let prop = D2D1_BITMAP_PROPERTIES {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_R8G8B8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: dpix,
            dpiY: dpiy,
        };
        let bitmap = unsafe {
            target
                .CreateBitmap(
                    D2D_SIZE_U {
                        width: image.width(),
                        height: image.height(),
                    },
                    Some(image.as_ptr().cast()),
                    image.width() * Rgba::<u8>::CHANNEL_COUNT as u32,
                    &prop,
                )
                .unwrap()
        };
        Self(bitmap)
    }

    pub fn size(&self) -> Size {
        let size = unsafe { self.0.GetSize() };
        Size::new(size.width as _, size.height as _)
    }
}
