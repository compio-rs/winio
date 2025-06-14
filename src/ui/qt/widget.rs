use std::{fmt::Debug, mem::ManuallyDrop, pin::Pin};

use cxx::{ExternType, UniquePtr, type_id};
pub(crate) use ffi::*;

use crate::{AsRawWindow, Point, RawWindow, Rect, Size};

pub struct Widget {
    widget: ManuallyDrop<UniquePtr<QWidget>>,
}

impl Widget {
    pub fn new(widget: UniquePtr<QWidget>) -> Self {
        Self {
            widget: ManuallyDrop::new(widget),
        }
    }

    pub(crate) unsafe fn drop_in_place(&mut self) {
        ManuallyDrop::drop(&mut self.widget);
    }

    pub(crate) fn as_ref(&self) -> &QWidget {
        &self.widget
    }

    pub(crate) fn pin_mut(&mut self) -> Pin<&mut QWidget> {
        self.widget.pin_mut()
    }

    pub fn is_visible(&self) -> bool {
        self.widget.isVisible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.widget.pin_mut().setVisible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.widget.isEnabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.widget.pin_mut().setEnabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        let s = self.widget.sizeHint();
        Size::new(s.width as _, s.height as _)
    }

    pub fn loc(&self) -> Point {
        Point::new(self.widget.x() as _, self.widget.y() as _)
    }

    pub fn set_loc(&mut self, p: Point) {
        self.widget.pin_mut().move_(p.x as _, p.y as _);
    }

    pub fn size(&self) -> Size {
        Size::new(self.widget.width() as _, self.widget.height() as _)
    }

    pub fn set_size(&mut self, s: Size) {
        self.widget.pin_mut().resize(s.width as _, s.height as _);
    }

    pub fn client_rect(&self) -> Rect {
        let geometry = self.widget.geometry();
        Rect::new(
            Point::new(geometry.x1 as _, geometry.y1 as _),
            Size::new(
                (geometry.x2 - geometry.x1) as _,
                (geometry.y2 - geometry.y1) as _,
            ),
        )
    }

    pub fn text(&self) -> String {
        widget_get_title(&self.widget)
    }

    pub fn set_text(&mut self, s: &str) {
        widget_set_title(self.widget.pin_mut(), s);
    }
}

impl AsRawWindow for Widget {
    fn as_raw_window(&self) -> RawWindow {
        self.widget
            .as_ref()
            .map(|p| p as *const _ as *mut _)
            .unwrap_or(std::ptr::null_mut())
    }
}

impl Debug for Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Widget").finish_non_exhaustive()
    }
}

#[repr(C)]
pub struct QSize {
    pub width: i32,
    pub height: i32,
}

unsafe impl ExternType for QSize {
    type Id = type_id!("QSize");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
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
        include!("winio/src/ui/qt/window.hpp");

        fn is_dark() -> bool;

        type QWidget;
        type QSize = super::QSize;
        type QRect = super::QRect;

        unsafe fn new_main_window(parent: *mut QWidget) -> UniquePtr<QWidget>;

        fn parentWidget(self: &QWidget) -> *mut QWidget;
        fn x(self: &QWidget) -> i32;
        fn y(self: &QWidget) -> i32;
        #[cxx_name = "move"]
        fn move_(self: Pin<&mut QWidget>, x: i32, y: i32);
        fn width(self: &QWidget) -> i32;
        fn height(self: &QWidget) -> i32;
        fn resize(self: Pin<&mut QWidget>, w: i32, h: i32);
        fn geometry(self: &QWidget) -> &QRect;
        fn sizeHint(self: &QWidget) -> QSize;
        fn update(self: Pin<&mut QWidget>);
        fn isVisible(self: &QWidget) -> bool;
        fn setVisible(self: Pin<&mut QWidget>, v: bool);
        fn isEnabled(self: &QWidget) -> bool;
        fn setEnabled(self: Pin<&mut QWidget>, v: bool);

        fn widget_get_title(w: &QWidget) -> String;
        fn widget_set_title(w: Pin<&mut QWidget>, s: &str);
    }
}
