use std::{
    cell::RefCell,
    io,
    mem::ManuallyDrop,
    pin::Pin,
    rc::{Rc, Weak},
};

use cxx::{ExternType, UniquePtr, type_id};

use super::Widget;
use crate::{
    BrushPen, Callback, Color, DrawingFont, HAlign, MouseButton, Point, Rect, Size,
    SolidColorBrush, VAlign,
};

pub struct Canvas {
    widget: Widget,
    on_paint: Callback,
    on_move: Callback<Point>,
    on_press: Callback<MouseButton>,
    on_release: Callback<MouseButton>,
}

impl Canvas {
    pub fn new(parent: &Widget) -> io::Result<Rc<Self>> {
        let mut widget = parent.pin_mut(ffi::new_canvas);
        let widget = Rc::new_cyclic(|this: &Weak<Self>| {
            unsafe {
                ffi::canvas_register_paint_event(
                    widget.pin_mut(),
                    Self::on_paint,
                    this.clone().into_raw().cast(),
                );
                ffi::canvas_register_move_event(
                    widget.pin_mut(),
                    Self::on_move,
                    this.clone().into_raw().cast(),
                );
                ffi::canvas_register_press_event(
                    widget.pin_mut(),
                    Self::on_press,
                    this.clone().into_raw().cast(),
                );
                ffi::canvas_register_release_event(
                    widget.pin_mut(),
                    Self::on_release,
                    this.clone().into_raw().cast(),
                );
            }
            Self {
                widget: Widget::new(widget),
                on_paint: Callback::new(),
                on_move: Callback::new(),
                on_press: Callback::new(),
                on_release: Callback::new(),
            }
        });
        Ok(widget)
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(self.widget.loc())
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.widget.set_loc(p);
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(self.widget.size())
    }

    pub fn set_size(&self, s: Size) -> io::Result<()> {
        self.widget.set_size(s);
        Ok(())
    }

    pub fn redraw(&self) -> io::Result<()> {
        self.widget.pin_mut(|w| w.repaint());
        Ok(())
    }

    fn on_paint(this: *const u8) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_paint.signal(());
        }
    }

    fn on_move(this: *const u8, x: i32, y: i32) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_move.signal(Point::new(x as _, y as _));
        }
    }

    fn on_press(this: *const u8, m: QtMouseButton) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_press.signal(m.into());
        }
    }

    fn on_release(this: *const u8, m: QtMouseButton) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_release.signal(m.into());
        }
    }

    pub async fn wait_redraw(&self) -> io::Result<DrawingContext> {
        self.on_paint.wait().await;
        Ok(DrawingContext {
            painter: RefCell::new(self.widget.pin_mut(ffi::canvas_new_painter)),
        })
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.on_press.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.on_release.wait().await
    }

    pub async fn wait_mouse_move(&self) -> io::Result<Point> {
        Ok(self.on_move.wait().await)
    }
}

pub struct DrawingContext {
    painter: RefCell<UniquePtr<ffi::QPainter>>,
}

fn set_brush(painter: Pin<&mut ffi::QPainter>, brush: impl Brush) {
    brush.set(painter);
}

fn set_pen(painter: Pin<&mut ffi::QPainter>, pen: impl Pen) {
    pen.set(painter);
}

impl DrawingContext {
    pub fn draw_ellipse(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        let mut painter = self.painter.borrow_mut();
        set_pen(painter.pin_mut(), pen);
        painter.pin_mut().drawEllipse(&QRectF(rect));
        Ok(())
    }

    pub fn draw_str(
        &self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) -> io::Result<()> {
        let text = text.as_ref();
        let mut painter = self.painter.borrow_mut();
        let size = ffi::painter_set_font(
            painter.pin_mut(),
            &font.family,
            font.size,
            font.italic,
            font.bold,
            text,
        )
        .0;
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

        set_brush(painter.pin_mut(), brush);
        ffi::painter_draw_text(painter.pin_mut(), QRectF(rect), text);
        Ok(())
    }
}

pub trait Brush {
    fn set(&self, painter: Pin<&mut ffi::QPainter>);
}

impl<B: Brush> Brush for &'_ B {
    fn set(&self, painter: Pin<&mut ffi::QPainter>) {
        (**self).set(painter)
    }
}

impl Brush for SolidColorBrush {
    fn set(&self, painter: Pin<&mut ffi::QPainter>) {
        ffi::painter_set_solid_brush(painter, self.color.into());
    }
}

pub trait Pen {
    fn set(&self, painter: Pin<&mut ffi::QPainter>);
}

impl<P: Pen> Pen for &'_ P {
    fn set(&self, painter: Pin<&mut ffi::QPainter>) {
        (**self).set(painter)
    }
}

impl Pen for BrushPen<SolidColorBrush> {
    fn set(&self, painter: Pin<&mut ffi::QPainter>) {
        ffi::painter_set_color_pen(painter, self.brush.color.into(), self.width);
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

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
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
pub struct QSizeF(Size);

unsafe impl ExternType for QSizeF {
    type Id = type_id!("QSizeF");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/canvas.hpp");

        type QWidget = crate::QWidget;
        type QtMouseButton = super::QtMouseButton;

        fn new_canvas(parent: Pin<&mut QWidget>) -> UniquePtr<QWidget>;
        unsafe fn canvas_register_paint_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
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
        type QSizeF = super::QSizeF;

        fn alpha(self: &QColor) -> i32;
        fn red(self: &QColor) -> i32;
        fn green(self: &QColor) -> i32;
        fn blue(self: &QColor) -> i32;

        fn drawEllipse(self: Pin<&mut QPainter>, rectangle: &QRectF);

        fn canvas_new_painter(w: Pin<&mut QWidget>) -> UniquePtr<QPainter>;
        fn painter_set_solid_brush(p: Pin<&mut QPainter>, c: QColor);
        fn painter_set_color_pen(p: Pin<&mut QPainter>, c: QColor, width: f64);
        fn painter_set_font(
            p: Pin<&mut QPainter>,
            family: &str,
            size: f64,
            italic: bool,
            bold: bool,
            text: &str,
        ) -> QSizeF;
        fn painter_draw_text(p: Pin<&mut QPainter>, rect: QRectF, text: &str);
    }
}
