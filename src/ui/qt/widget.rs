use std::{cell::RefCell, pin::Pin};

use cxx::{ExternType, UniquePtr, type_id};
pub(crate) use ffi::*;

use crate::{Point, Rect, Size};

#[repr(C)]
#[doc(hidden)]
pub struct QRect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

unsafe impl ExternType for QRect {
    type Id = type_id!("QRect");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/widget.hpp");

        fn is_dark() -> bool;

        type QWidget;
        type QRect = super::QRect;

        fn x(self: &QWidget) -> i32;
        fn y(self: &QWidget) -> i32;
        #[cxx_name = "move"]
        fn move_(self: Pin<&mut QWidget>, x: i32, y: i32);
        fn width(self: &QWidget) -> i32;
        fn height(self: &QWidget) -> i32;
        fn resize(self: Pin<&mut QWidget>, w: i32, h: i32);
        fn geometry(self: &QWidget) -> &QRect;

        fn widget_get_title(w: &QWidget) -> String;
        fn widget_set_title(w: Pin<&mut QWidget>, s: &str);

        fn new_main_window() -> UniquePtr<QWidget>;
        unsafe fn main_window_register_resize_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        );
        unsafe fn main_window_register_move_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        );
        unsafe fn main_window_register_close_event(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8) -> bool,
            data: *const u8,
        );

        fn new_push_button(parent: Pin<&mut QWidget>) -> UniquePtr<QWidget>;
        unsafe fn push_button_connect_clicked(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
    }
}

pub struct Widget {
    widget: RefCell<UniquePtr<QWidget>>,
}

impl Widget {
    pub fn new(widget: UniquePtr<QWidget>) -> Self {
        Self {
            widget: RefCell::new(widget),
        }
    }

    pub(crate) fn pin_mut<T>(&self, f: impl Fn(Pin<&mut QWidget>) -> T) -> T {
        f(self.widget.borrow_mut().pin_mut())
    }

    pub fn loc(&self) -> Point {
        let widget = self.widget.borrow();
        Point::new(widget.x() as _, widget.y() as _)
    }

    pub fn set_loc(&self, p: Point) {
        self.widget.borrow_mut().pin_mut().move_(p.x as _, p.y as _);
    }

    pub fn size(&self) -> Size {
        let widget = self.widget.borrow();
        Size::new(widget.width() as _, widget.height() as _)
    }

    pub fn set_size(&self, s: Size) {
        self.widget
            .borrow_mut()
            .pin_mut()
            .resize(s.width as _, s.height as _);
    }

    pub fn client_rect(&self) -> Rect {
        let widget = self.widget.borrow();
        let geometry = widget.geometry();
        Rect::new(
            Point::new(geometry.x1 as _, geometry.y1 as _),
            Size::new(geometry.x2 as _, geometry.y2 as _),
        )
    }

    pub fn text(&self) -> String {
        widget_get_title(&self.widget.borrow())
    }

    pub fn set_text(&self, s: &str) {
        let mut widget = self.widget.borrow_mut();
        widget_set_title(widget.pin_mut(), s);
    }
}
