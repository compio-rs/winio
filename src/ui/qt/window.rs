use std::{
    io,
    rc::{Rc, Weak},
};

use super::Widget;
use crate::{Callback, Point, Size};

pub struct Window {
    widget: Widget,
    on_close: Callback<()>,
}

impl Window {
    pub fn new() -> io::Result<Rc<Self>> {
        let mut widget = super::new_main_window();
        let widget = Rc::new_cyclic(|this: &Weak<Self>| {
            unsafe {
                super::main_window_close_event(
                    widget.pin_mut(),
                    Self::on_close,
                    this.clone().into_raw().cast(),
                );
            }
            Self {
                widget: Widget::new(widget),
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

    fn on_close(this: *const u8) -> bool {
        let this = unsafe { Weak::<Self>::from_raw(this.cast()) };
        if let Some(this) = this.upgrade() {
            if !this.on_close.signal(()) {
                return true;
            }
        }
        false
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }
}
