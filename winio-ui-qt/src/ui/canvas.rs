use std::{mem::MaybeUninit, pin::Pin};

use cxx::{ExternType, UniquePtr, type_id};
pub(crate) use ffi::QWidget;
use image::{DynamicImage, Pixel, Rgb, Rgba};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{
    BrushPen, Color, DrawingFont, HAlign, LinearGradientBrush, MouseButton, Point,
    RadialGradientBrush, Rect, RectBox, RelativeToLogical, Size, SolidColorBrush, VAlign, Vector,
};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
pub struct Canvas {
    on_move: Box<Callback<Point>>,
    on_press: Box<Callback<MouseButton>>,
    on_release: Box<Callback<MouseButton>>,
    on_wheel: Box<Callback<Vector>>,
    widget: Widget<ffi::QWidget>,
}

#[inherit_methods(from = "self.widget")]
impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_canvas(parent.as_window().as_qt()) };
        widget.pin_mut().setVisible(true);
        let on_move = Box::new(Callback::new());
        let on_press = Box::new(Callback::new());
        let on_release = Box::new(Callback::new());
        let on_wheel = Box::new(Callback::new());
        unsafe {
            ffi::canvas_register_move_event(
                widget.pin_mut(),
                Self::on_move,
                on_move.as_ref() as *const _ as _,
            );
            ffi::canvas_register_press_event(
                widget.pin_mut(),
                Self::on_press,
                on_press.as_ref() as *const _ as _,
            );
            ffi::canvas_register_release_event(
                widget.pin_mut(),
                Self::on_release,
                on_release.as_ref() as *const _ as _,
            );
            ffi::canvas_register_wheel_event(
                widget.pin_mut(),
                Self::on_wheel,
                on_wheel.as_ref() as *const _ as _,
            );
        }
        Self {
            on_move,
            on_press,
            on_release,
            on_wheel,
            widget: Widget::new(widget),
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

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

    pub fn context(&mut self) -> DrawingContext<'_> {
        DrawingContext {
            painter: ffi::canvas_new_painter(self.widget.pin_mut()),
            size: self.widget.size(),
            canvas: self,
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
}

winio_handle::impl_as_widget!(Canvas, widget);

pub struct DrawingContext<'a> {
    painter: UniquePtr<ffi::QPainter>,
    size: Size,
    canvas: &'a mut Canvas,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        self.painter.pin_mut().end();
        self.canvas.widget.pin_mut().update();
    }
}

#[inline]
fn to_trans(rect: Rect) -> RelativeToLogical {
    RelativeToLogical::scale(rect.size.width, rect.size.height)
        .then_translate(rect.origin.to_vector())
}

fn drawing_angle(angle: f64) -> i32 {
    (-angle * 180.0 / std::f64::consts::PI * 16.0).round() as i32
}

impl DrawingContext<'_> {
    fn set_brush(&mut self, brush: impl Brush, rect: Rect) {
        self.painter
            .pin_mut()
            .setBrush(&brush.create(to_trans(rect)));
        self.painter.pin_mut().setPen_color(&QColor::transparent());
    }

    fn set_pen(&mut self, pen: impl Pen, rect: Rect) {
        self.painter.pin_mut().setPen(&pen.create(to_trans(rect)));
        self.painter
            .pin_mut()
            .setBrush(&ffi::new_brush(&QColor::transparent()));
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) {
        let rect = path.0.boundingRect();
        self.set_pen(pen, rect.0);
        self.painter.pin_mut().drawPath(&path.0);
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) {
        let rect = path.0.boundingRect();
        self.set_brush(brush, rect.0);
        self.painter.pin_mut().drawPath(&path.0);
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.set_pen(pen, rect);
        self.painter.pin_mut().drawArc(
            &QRectF(rect),
            drawing_angle(start),
            drawing_angle(end - start),
        );
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.set_pen(pen, rect);
        self.painter.pin_mut().drawPie(
            &QRectF(rect),
            drawing_angle(start),
            drawing_angle(end - start),
        );
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        self.set_brush(brush, rect);
        self.painter.pin_mut().drawPie(
            &QRectF(rect),
            drawing_angle(start),
            drawing_angle(end - start),
        );
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        self.set_pen(pen, rect);
        self.painter.pin_mut().drawEllipse(&QRectF(rect));
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        self.set_brush(brush, rect);
        self.painter.pin_mut().drawEllipse(&QRectF(rect));
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        self.set_pen(pen, rect);
        self.painter
            .pin_mut()
            .drawLine(&QPointF(start), &QPointF(end));
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        self.set_pen(pen, rect);
        self.painter.pin_mut().drawRect(&QRectF(rect));
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        self.set_brush(brush, rect);
        self.painter.pin_mut().drawRect(&QRectF(rect));
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        self.set_pen(pen, rect);
        self.painter.pin_mut().drawRoundedRect(
            &QRectF(rect),
            round.width,
            round.height,
            QtSizeMode::AbsoluteSize,
        );
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        self.set_brush(brush, rect);
        self.painter.pin_mut().drawRoundedRect(
            &QRectF(rect),
            round.width,
            round.height,
            QtSizeMode::AbsoluteSize,
        );
    }

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str) {
        ffi::painter_set_font(
            self.painter.pin_mut(),
            &font.family,
            font.size,
            font.italic,
            font.bold,
        );
        let rect = Rect::new(Point::zero(), self.size);
        let size = ffi::painter_measure_text(self.painter.pin_mut(), QRectF(rect), text).0;
        let mut rect = Rect::new(pos, size);
        match font.halign {
            HAlign::Center => rect.origin.x -= rect.width() / 2.0,
            HAlign::Right => rect.origin.x -= rect.width(),
            _ => {}
        }
        match font.valign {
            VAlign::Center => rect.origin.y -= rect.height() / 2.0,
            VAlign::Bottom => rect.origin.y -= rect.height(),
            _ => {}
        }

        self.set_pen(BrushPen::new(brush, 1.0), rect);
        ffi::painter_draw_text(self.painter.pin_mut(), QRectF(rect), text);
    }

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage {
        DrawingImage::new(image)
    }

    pub fn draw_image(&mut self, image: &DrawingImage, rect: Rect, clip: Option<Rect>) {
        let clip = clip.unwrap_or_else(|| Rect::new(Point::zero(), image.size()));
        ffi::painter_draw_image(
            self.painter.pin_mut(),
            &QRectF(rect),
            &image.pixmap,
            &QRectF(clip),
        );
    }

    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder {
        DrawingPathBuilder::new(start)
    }
}

pub struct DrawingPath(UniquePtr<ffi::QPainterPath>);

pub struct DrawingPathBuilder(UniquePtr<ffi::QPainterPath>);

impl DrawingPathBuilder {
    fn new(start: Point) -> Self {
        let mut ptr = ffi::new_path();
        ptr.pin_mut().moveTo(start.x, start.y);
        Self(ptr)
    }

    pub fn add_line(&mut self, p: Point) {
        self.0.pin_mut().lineTo(p.x, p.y);
    }

    pub fn add_arc(&mut self, center: Point, radius: Size, start: f64, end: f64, clockwise: bool) {
        self.0.pin_mut().arcTo(
            center.x - radius.width,
            center.y - radius.height,
            radius.width * 2.0,
            radius.height * 2.0,
            start,
            if clockwise { start - end } else { end - start } / std::f64::consts::PI * 180.0,
        );
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) {
        self.0.pin_mut().cubicTo(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
    }

    pub fn build(mut self, close: bool) -> DrawingPath {
        if close {
            self.0.pin_mut().closeSubpath();
        }
        DrawingPath(self.0)
    }
}

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn create(&self, trans: RelativeToLogical) -> QBrush;
}

impl<B: Brush> Brush for &'_ B {
    fn create(&self, trans: RelativeToLogical) -> QBrush {
        (**self).create(trans)
    }
}

impl Brush for SolidColorBrush {
    fn create(&self, _trans: RelativeToLogical) -> QBrush {
        ffi::new_brush(&self.color.into())
    }
}

impl Brush for LinearGradientBrush {
    fn create(&self, trans: RelativeToLogical) -> QBrush {
        let mut g = ffi::new_gradient_linear(
            QPointF(Point::new(self.start.x, self.start.y)),
            QPointF(Point::new(self.end.x, self.end.y)),
        );
        for stop in &self.stops {
            g.pin_mut().setColorAt(stop.pos, &QColor::from(stop.color));
        }
        let mut brush = ffi::new_brush_gradient(&g);
        brush_set_transform(Pin::new(&mut brush), trans);
        brush
    }
}

impl Brush for RadialGradientBrush {
    fn create(&self, trans: RelativeToLogical) -> QBrush {
        let trans = trans.then_scale(1.0, self.radius.height / self.radius.width);
        let mut g = ffi::new_gradient_radial(
            QPointF(Point::new(self.center.x, self.center.y)),
            self.radius.width,
            QPointF(Point::new(self.origin.x, self.origin.y)),
        );
        for stop in &self.stops {
            g.pin_mut().setColorAt(stop.pos, &QColor::from(stop.color));
        }
        let mut brush = ffi::new_brush_gradient(&g);
        brush_set_transform(Pin::new(&mut brush), trans);
        brush
    }
}

fn brush_set_transform(b: Pin<&mut ffi::QBrush>, trans: RelativeToLogical) {
    ffi::brush_set_transform(
        b, trans.m11, trans.m12, trans.m21, trans.m22, trans.m31, trans.m32,
    );
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn create(&self, trans: RelativeToLogical) -> QPen;
}

impl<P: Pen> Pen for &'_ P {
    fn create(&self, trans: RelativeToLogical) -> QPen {
        (**self).create(trans)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create(&self, trans: RelativeToLogical) -> QPen {
        let brush = self.brush.create(trans);
        ffi::new_pen(&brush, self.width)
    }
}

pub struct DrawingImage {
    #[allow(dead_code)]
    buffer: Vec<u8>,
    pixmap: UniquePtr<ffi::QImage>,
}

impl DrawingImage {
    fn new(image: DynamicImage) -> Self {
        let width = image.width();
        let height = image.height();
        let (format, buffer, count) = match image {
            DynamicImage::ImageRgb8(_) => (
                QImageFormat::RGB888,
                image.into_bytes(),
                Rgb::<u8>::CHANNEL_COUNT,
            ),
            DynamicImage::ImageRgba8(_) => (
                QImageFormat::RGBA8888,
                image.into_bytes(),
                Rgba::<u8>::CHANNEL_COUNT,
            ),
            DynamicImage::ImageRgba16(_) => (
                QImageFormat::RGBA64,
                image.into_bytes(),
                Rgba::<u16>::CHANNEL_COUNT * 2,
            ),
            DynamicImage::ImageRgba32F(_) => (
                QImageFormat::RGBA32FPx4,
                image.into_bytes(),
                Rgba::<f32>::CHANNEL_COUNT * 4,
            ),
            _ => (
                QImageFormat::RGBA32FPx4,
                DynamicImage::ImageRgba32F(image.into_rgba32f()).into_bytes(),
                Rgba::<f32>::CHANNEL_COUNT * 4,
            ),
        };
        let pixmap = unsafe {
            ffi::new_image(
                width as _,
                height as _,
                (width * count as u32) as _,
                buffer.as_ptr(),
                format,
            )
        };
        Self { buffer, pixmap }
    }

    pub fn size(&self) -> Size {
        Size::new(self.pixmap.width() as _, self.pixmap.height() as _)
    }
}

/// Get the accent color.
pub fn accent_color() -> Option<Color> {
    QColor::accent().map(From::from)
}

#[repr(i32)]
#[non_exhaustive]
#[allow(dead_code, clippy::enum_variant_names)]
pub(crate) enum QtMouseButton {
    NoButton     = 0x00000000,
    LeftButton   = 0x00000001,
    RightButton  = 0x00000002,
    MiddleButton = 0x00000004,
}

impl From<QtMouseButton> for MouseButton {
    fn from(value: QtMouseButton) -> Self {
        match value {
            QtMouseButton::LeftButton => MouseButton::Left,
            QtMouseButton::MiddleButton => MouseButton::Middle,
            QtMouseButton::RightButton => MouseButton::Right,
            _ => MouseButton::Other,
        }
    }
}

unsafe impl ExternType for QtMouseButton {
    type Id = type_id!("QtMouseButton");
    type Kind = cxx::kind::Trivial;
}

#[repr(i32)]
#[allow(dead_code)]
pub(crate) enum QtSizeMode {
    AbsoluteSize,
    RelativeSize,
}

unsafe impl ExternType for QtSizeMode {
    type Id = type_id!("QtSizeMode");
    type Kind = cxx::kind::Trivial;
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
#[allow(dead_code)]
pub(crate) enum Spec {
    Invalid,
    Rgb,
    Hsv,
    Cmyk,
    Hsl,
    ExtendedRgb,
}

#[repr(C)]
pub(crate) struct QColor {
    cspec: Spec,
    ct: [u16; 5],
}

unsafe impl ExternType for QColor {
    type Id = type_id!("QColor");
    type Kind = cxx::kind::Trivial;
}

const fn is_rgba_value(r: i32, g: i32, b: i32, a: i32) -> bool {
    ((r as u32) <= 255) && ((g as u32) <= 255) && ((b as u32) <= 255) && ((a as u32) <= 255)
}

impl QColor {
    pub const fn new(r: i32, g: i32, b: i32, a: i32) -> Self {
        let cspec = if is_rgba_value(r, g, b, a) {
            Spec::Rgb
        } else {
            Spec::Invalid
        };
        if let Spec::Rgb = cspec {
            Self {
                cspec,
                ct: [
                    (a * 0x0101) as u16,
                    (r * 0x0101) as u16,
                    (g * 0x0101) as u16,
                    (b * 0x0101) as u16,
                    0,
                ],
            }
        } else {
            Self { cspec, ct: [0; 5] }
        }
    }

    pub fn transparent() -> Self {
        let mut c = Self {
            cspec: Spec::Invalid,
            ct: [0; 5],
        };
        ffi::color_transparent(Pin::new(&mut c));
        c
    }

    pub fn accent() -> Option<Self> {
        let mut c = Self {
            cspec: Spec::Invalid,
            ct: [0; 5],
        };
        if ffi::color_accent(Pin::new(&mut c)) {
            Some(c)
        } else {
            None
        }
    }
}

impl From<Color> for QColor {
    fn from(value: Color) -> Self {
        Self::new(value.r as _, value.g as _, value.b as _, value.a as _)
    }
}

impl From<QColor> for Color {
    fn from(value: QColor) -> Self {
        Self::new(
            value.red() as _,
            value.green() as _,
            value.blue() as _,
            value.alpha() as _,
        )
    }
}

#[repr(transparent)]
pub(crate) struct QRectF(Rect);

unsafe impl ExternType for QRectF {
    type Id = type_id!("QRectF");
    type Kind = cxx::kind::Trivial;
}

#[repr(transparent)]
pub(crate) struct QPointF(Point);

unsafe impl ExternType for QPointF {
    type Id = type_id!("QPointF");
    type Kind = cxx::kind::Trivial;
}

#[repr(transparent)]
pub(crate) struct QSizeF(Size);

unsafe impl ExternType for QSizeF {
    type Id = type_id!("QSizeF");
    type Kind = cxx::kind::Trivial;
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
pub(crate) enum QImageFormat {
    RGB888     = 13,
    RGBA8888   = 17,
    RGBA64     = 26,
    RGBA32FPx4 = 34,
}

unsafe impl ExternType for QImageFormat {
    type Id = type_id!("QImageFormat");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
#[doc(hidden)]
pub struct QPen {
    _data: MaybeUninit<usize>,
}

unsafe impl ExternType for QPen {
    type Id = type_id!("QPen");
    type Kind = cxx::kind::Trivial;
}

impl Drop for QPen {
    fn drop(&mut self) {
        ffi::pen_drop(Pin::new(self));
    }
}

#[repr(C)]
#[doc(hidden)]
pub struct QBrush {
    _data: MaybeUninit<usize>,
}

unsafe impl ExternType for QBrush {
    type Id = type_id!("QBrush");
    type Kind = cxx::kind::Trivial;
}

impl Drop for QBrush {
    fn drop(&mut self) {
        ffi::brush_drop(Pin::new(self));
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/canvas.hpp");

        type QWidget;
        type QtMouseButton = super::QtMouseButton;

        unsafe fn new_canvas(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn canvas_register_move_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        );
        unsafe fn canvas_register_press_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, QtMouseButton),
            data: *const u8,
        );
        unsafe fn canvas_register_release_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, QtMouseButton),
            data: *const u8,
        );
        unsafe fn canvas_register_wheel_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        );

        type QPainter;
        type QColor = super::QColor;
        type QRectF = super::QRectF;
        type QPointF = super::QPointF;
        type QSizeF = super::QSizeF;
        type QtSizeMode = super::QtSizeMode;

        fn alpha(self: &QColor) -> i32;
        fn red(self: &QColor) -> i32;
        fn green(self: &QColor) -> i32;
        fn blue(self: &QColor) -> i32;

        fn drawArc(self: Pin<&mut QPainter>, rectangle: &QRectF, start: i32, span: i32);
        fn drawPie(self: Pin<&mut QPainter>, rectangle: &QRectF, start: i32, span: i32);
        fn drawEllipse(self: Pin<&mut QPainter>, rectangle: &QRectF);
        fn drawLine(self: Pin<&mut QPainter>, p1: &QPointF, p2: &QPointF);
        fn drawRect(self: Pin<&mut QPainter>, rectangle: &QRectF);
        fn drawRoundedRect(
            self: Pin<&mut QPainter>,
            rectangle: &QRectF,
            xr: f64,
            yr: f64,
            mode: QtSizeMode,
        );
        fn drawPath(self: Pin<&mut QPainter>, path: &QPainterPath);

        fn end(self: Pin<&mut QPainter>) -> bool;

        fn canvas_new_painter(w: Pin<&mut QWidget>) -> UniquePtr<QPainter>;
        fn painter_set_font(
            p: Pin<&mut QPainter>,
            family: &str,
            size: f64,
            italic: bool,
            bold: bool,
        );
        fn painter_measure_text(p: Pin<&mut QPainter>, rect: QRectF, text: &str) -> QSizeF;
        fn painter_draw_text(p: Pin<&mut QPainter>, rect: QRectF, text: &str);

        type QBrush = super::QBrush;
        type QPen = super::QPen;

        fn setBrush(self: Pin<&mut QPainter>, brush: &QBrush);
        fn setPen(self: Pin<&mut QPainter>, pen: &QPen);
        #[rust_name = "setPen_color"]
        fn setPen(self: Pin<&mut QPainter>, color: &QColor);

        fn color_transparent(c: Pin<&mut QColor>);
        fn color_accent(c: Pin<&mut QColor>) -> bool;
        fn new_brush(c: &QColor) -> QBrush;
        fn new_pen(b: &QBrush, width: f64) -> QPen;

        fn brush_drop(b: Pin<&mut QBrush>);
        fn pen_drop(p: Pin<&mut QPen>);

        type QGradient;

        fn new_gradient_linear(start: QPointF, end: QPointF) -> UniquePtr<QGradient>;
        fn new_gradient_radial(
            center: QPointF,
            radius: f64,
            origin: QPointF,
        ) -> UniquePtr<QGradient>;
        fn setColorAt(self: Pin<&mut QGradient>, pos: f64, c: &QColor);

        fn new_brush_gradient(g: &QGradient) -> QBrush;
        fn brush_set_transform(
            b: Pin<&mut QBrush>,
            m11: f64,
            m12: f64,
            m21: f64,
            m22: f64,
            m31: f64,
            m32: f64,
        );

        type QImage;
        type QImageFormat = super::QImageFormat;

        unsafe fn new_image(
            width: i32,
            height: i32,
            stride: i32,
            bits: *const u8,
            format: QImageFormat,
        ) -> UniquePtr<QImage>;
        fn painter_draw_image(
            p: Pin<&mut QPainter>,
            target: &QRectF,
            image: &QImage,
            source: &QRectF,
        );

        fn width(self: &QImage) -> i32;
        fn height(self: &QImage) -> i32;

        type QPainterPath;

        fn new_path() -> UniquePtr<QPainterPath>;

        fn moveTo(self: Pin<&mut QPainterPath>, x: f64, y: f64);
        fn lineTo(self: Pin<&mut QPainterPath>, x: f64, y: f64);
        fn arcTo(
            self: Pin<&mut QPainterPath>,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
            start: f64,
            sweep: f64,
        );
        fn cubicTo(
            self: Pin<&mut QPainterPath>,
            x1: f64,
            y1: f64,
            x2: f64,
            y2: f64,
            x3: f64,
            y3: f64,
        );
        fn closeSubpath(self: Pin<&mut QPainterPath>);
        fn boundingRect(self: &QPainterPath) -> QRectF;
    }
}
