use std::{cell::RefCell, pin::Pin};

use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/widget.hpp");

        type QWidget;

        fn x(self: &QWidget) -> i32;
        fn y(self: &QWidget) -> i32;
        #[cxx_name = "move"]
        fn move_(self: Pin<&mut QWidget>, x: i32, y: i32);
        fn width(self: &QWidget) -> i32;
        fn height(self: &QWidget) -> i32;
        fn resize(self: Pin<&mut QWidget>, w: i32, h: i32);

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

pub(crate) use ffi::*;

use crate::{Point, Size};

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
}
