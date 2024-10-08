use super::RawWindow;
use crate::{
    AsRawWindow, Point, Size,
    ui::{Callback, Widget},
};

pub struct Window {
    on_resize: Box<Callback<Size>>,
    on_move: Box<Callback<Point>>,
    on_close: Box<Callback<()>>,
    widget: Widget,
}

impl Window {
    pub fn new() -> Self {
        let mut widget = super::new_main_window();
        widget.pin_mut().show();
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

    pub fn client_size(&self) -> Size {
        self.widget.client_rect().size
    }

    pub fn text(&self) -> String {
        self.widget.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s.as_ref());
    }

    fn on_resize(c: *const u8, width: i32, height: i32) {
        let c = c as *const Callback<Size>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(Size::new(width as _, height as _));
        }
    }

    fn on_move(c: *const u8, x: i32, y: i32) {
        let c = c as *const Callback<Point>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(Point::new(x as _, y as _));
        }
    }

    fn on_close(c: *const u8) -> bool {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            if !c.signal(()) {
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
        self.widget.as_raw_window()
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/window.hpp");

        type QWidget = crate::ui::QWidget;

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
    }
}
