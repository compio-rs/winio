use std::pin::Pin;

use cxx::{ExternType, UniquePtr, type_id};

use crate::{
    AsRawWindow, AsWindow, BrushPen, Color, DrawingFont, HAlign, LinearGradientBrush, MouseButton,
    Point, RadialGradientBrush, Rect, RectBox, RelativeToLogical, Size, SolidColorBrush, VAlign,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct Canvas {
    on_move: Box<Callback<Point>>,
    on_press: Box<Callback<MouseButton>>,
    on_release: Box<Callback<MouseButton>>,
    widget: Widget,
}

impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_canvas(parent.as_window().as_raw_window()) };
        widget.pin_mut().show();
        let on_move = Box::new(Callback::new());
        let on_press = Box::new(Callback::new());
        let on_release = Box::new(Callback::new());
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
        }
        Self {
            on_move,
            on_press,
            on_release,
            widget: Widget::new(widget),
        }
    }

    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.widget.size()
    }

    pub fn set_size(&mut self, s: Size) {
        self.widget.set_size(s);
    }

    fn on_move(c: *const u8, x: i32, y: i32) {
        let c = c as *const Callback<Point>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(Point::new(x as _, y as _));
        }
    }

    fn on_press(c: *const u8, m: QtMouseButton) {
        let c = c as *const Callback<MouseButton>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(m.into());
        }
    }

    fn on_release(c: *const u8, m: QtMouseButton) {
        let c = c as *const Callback<MouseButton>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(m.into());
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
}

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

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.set_pen(pen, rect);
        self.painter.pin_mut().drawArc(
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
}

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QBrush>;
}

impl<B: Brush> Brush for &'_ B {
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QBrush> {
        (**self).create(trans)
    }
}

impl Brush for SolidColorBrush {
    fn create(&self, _trans: RelativeToLogical) -> UniquePtr<ffi::QBrush> {
        ffi::new_brush(&self.color.into())
    }
}

impl Brush for LinearGradientBrush {
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QBrush> {
        let mut g = ffi::new_gradient_linear(
            QPointF(Point::new(self.start.x, self.start.y)),
            QPointF(Point::new(self.end.x, self.end.y)),
        );
        for stop in &self.stops {
            g.pin_mut().setColorAt(stop.pos, &QColor::from(stop.color));
        }
        let mut brush = ffi::new_brush_gradient(&g);
        brush_set_transform(brush.pin_mut(), trans);
        brush
    }
}

impl Brush for RadialGradientBrush {
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QBrush> {
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
        brush_set_transform(brush.pin_mut(), trans);
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
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QPen>;
}

impl<P: Pen> Pen for &'_ P {
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QPen> {
        (**self).create(trans)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create(&self, trans: RelativeToLogical) -> UniquePtr<ffi::QPen> {
        let brush = self.brush.create(trans);
        ffi::new_pen(&brush, self.width)
    }
}

#[repr(i32)]
#[non_exhaustive]
#[allow(clippy::enum_variant_names)]
pub enum QtMouseButton {
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
pub enum QtSizeMode {
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
pub enum Spec {
    Invalid,
    Rgb,
    Hsv,
    Cmyk,
    Hsl,
    ExtendedRgb,
}

#[repr(C)]
pub struct QColor {
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
pub struct QRectF(Rect);

unsafe impl ExternType for QRectF {
    type Id = type_id!("QRectF");
    type Kind = cxx::kind::Trivial;
}

#[repr(transparent)]
pub struct QPointF(Point);

unsafe impl ExternType for QPointF {
    type Id = type_id!("QPointF");
    type Kind = cxx::kind::Trivial;
}

#[repr(transparent)]
pub struct QSizeF(Size);

unsafe impl ExternType for QSizeF {
    type Id = type_id!("QSizeF");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/canvas.hpp");

        type QWidget = crate::ui::QWidget;
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

        type QBrush;
        type QPen;

        fn setBrush(self: Pin<&mut QPainter>, brush: &QBrush);
        fn setPen(self: Pin<&mut QPainter>, pen: &QPen);
        #[rust_name = "setPen_color"]
        fn setPen(self: Pin<&mut QPainter>, color: &QColor);

        fn color_transparent(c: Pin<&mut QColor>);
        fn new_brush(c: &QColor) -> UniquePtr<QBrush>;
        fn new_pen(b: &QBrush, width: f64) -> UniquePtr<QPen>;

        type QGradient;

        fn new_gradient_linear(start: QPointF, end: QPointF) -> UniquePtr<QGradient>;
        fn new_gradient_radial(
            center: QPointF,
            radius: f64,
            origin: QPointF,
        ) -> UniquePtr<QGradient>;
        fn setColorAt(self: Pin<&mut QGradient>, pos: f64, c: &QColor);

        fn new_brush_gradient(g: &QGradient) -> UniquePtr<QBrush>;
        fn brush_set_transform(
            b: Pin<&mut QBrush>,
            m11: f64,
            m12: f64,
            m21: f64,
            m22: f64,
            m31: f64,
            m32: f64,
        );
    }
}
