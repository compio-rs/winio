use std::{fmt::Debug, pin::Pin};

use cxx::UniquePtr;
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWindow, AsWindow, BorrowedContainer, BorrowedWindow,
    RawContainer, RawWindow,
};
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, StaticCastTo, ui::impl_static_cast};

pub struct Window {
    on_resize: Box<Callback<Size>>,
    on_move: Box<Callback<Point>>,
    on_close: Box<Callback<()>>,
    widget: UniquePtr<ffi::QMainWindow>,
}

impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut widget = ffi::new_main_window();
        let on_resize = Box::new(Callback::new());
        let on_move = Box::new(Callback::new());
        let on_close = Box::new(Callback::new());
        unsafe {
            ffi::main_window_register_resize_event(
                widget.pin_mut(),
                Self::on_resize,
                on_resize.as_ref() as *const _ as _,
            );
            ffi::main_window_register_move_event(
                widget.pin_mut(),
                Self::on_move,
                on_move.as_ref() as *const _ as _,
            );
            ffi::main_window_register_close_event(
                widget.pin_mut(),
                Self::on_close,
                on_close.as_ref() as *const _ as _,
            );
        }
        Self {
            on_resize,
            on_move,
            on_close,
            widget,
        }
    }

    fn as_ref_qwidget(&self) -> &ffi::QWidget {
        (*self.widget).static_cast()
    }

    fn pin_mut_qwidget(&mut self) -> Pin<&mut ffi::QWidget> {
        self.widget.pin_mut().static_cast_mut()
    }

    pub fn is_visible(&self) -> bool {
        self.as_ref_qwidget().isVisible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.pin_mut_qwidget().setVisible(v);
    }

    pub fn loc(&self) -> Point {
        Point::new(
            self.as_ref_qwidget().x() as _,
            self.as_ref_qwidget().y() as _,
        )
    }

    pub fn set_loc(&mut self, p: Point) {
        self.pin_mut_qwidget().move_(p.x as _, p.y as _);
    }

    pub fn size(&self) -> Size {
        Size::new(
            self.as_ref_qwidget().width() as _,
            self.as_ref_qwidget().height() as _,
        )
    }

    pub fn set_size(&mut self, s: Size) {
        self.pin_mut_qwidget().resize(s.width as _, s.height as _);
    }

    pub fn client_size(&self) -> Size {
        let geometry = self.as_ref_qwidget().geometry();
        Size::new(
            (geometry.x2 - geometry.x1) as _,
            (geometry.y2 - geometry.y1) as _,
        )
    }

    pub fn text(&self) -> String {
        self.as_ref_qwidget().windowTitle().into()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.pin_mut_qwidget().setWindowTitle(&s.as_ref().into());
    }

    fn on_resize(c: *const u8, width: i32, height: i32) {
        let c = c as *const Callback<Size>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(Size::new(width as _, height as _));
        }
    }

    fn on_move(c: *const u8, x: i32, y: i32) {
        let c = c as *const Callback<Point>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(Point::new(x as _, y as _));
        }
    }

    fn on_close(c: *const u8) -> bool {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            if !c.signal::<GlobalRuntime>(()) {
                return true;
            }
        }
        false
    }

    pub async fn wait_size(&self) {
        self.on_resize.wait().await;
    }

    pub async fn wait_move(&self) {
        self.on_move.wait().await;
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.as_raw_window()) }
    }
}

impl AsRawContainer for Window {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

impl AsContainer for Window {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}

impl_static_cast!(ffi::QMainWindow, ffi::QWidget);

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window").finish_non_exhaustive()
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/window.hpp");

        type QWidget = crate::ui::QWidget;
        type QMainWindow;

        fn new_main_window() -> UniquePtr<QMainWindow>;

        unsafe fn main_window_register_resize_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        );
        unsafe fn main_window_register_move_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        );
        unsafe fn main_window_register_close_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8) -> bool,
            data: *const u8,
        );
    }
}
