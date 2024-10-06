use std::{
    io,
    mem::ManuallyDrop,
    ops::Deref,
    rc::{Rc, Weak},
};

use super::Widget;
use crate::{Callback, Point, Size};

pub struct Window {
    widget: Widget,
    on_resize: Callback<Size>,
    on_move: Callback<Point>,
    on_close: Callback<()>,
}

impl Window {
    pub fn new() -> io::Result<Rc<Self>> {
        let mut widget = super::new_main_window();
        let widget = Rc::new_cyclic(move |this: &Weak<Self>| {
            unsafe {
                super::main_window_register_resize_event(
                    widget.pin_mut(),
                    Self::on_resize,
                    this.clone().into_raw().cast(),
                );
                super::main_window_register_move_event(
                    widget.pin_mut(),
                    Self::on_move,
                    this.clone().into_raw().cast(),
                );
                super::main_window_register_close_event(
                    widget.pin_mut(),
                    Self::on_close,
                    this.clone().into_raw().cast(),
                );
            }
            Self {
                widget: Widget::new(widget),
                on_resize: Callback::new(),
                on_move: Callback::new(),
                on_close: Callback::new(),
            }
        });
        Ok(widget)
    }

    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    pub fn set_loc(&self, p: Point) {
        self.widget.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.widget.size()
    }

    pub fn set_size(&self, s: Size) {
        self.widget.set_size(s);
    }

    fn on_resize(this: *const u8, width: i32, height: i32) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_resize.signal(Size::new(width as _, height as _));
        }
    }

    fn on_move(this: *const u8, x: i32, y: i32) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_move.signal(Point::new(x as _, y as _));
        }
    }

    fn on_close(this: *const u8) -> bool {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            if !this.on_close.signal(()) {
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

impl Deref for Window {
    type Target = Widget;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}
