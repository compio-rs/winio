use std::{
    io,
    mem::ManuallyDrop,
    rc::{Rc, Weak},
};

use crate::{Callback, Point, Size, Widget};

pub struct Button {
    widget: Widget,
    on_click: Callback,
}

impl Button {
    pub fn new(parent: &Widget) -> io::Result<Rc<Self>> {
        let mut widget = parent.pin_mut(super::new_push_button);
        let widget = Rc::new_cyclic(move |this: &Weak<Self>| {
            unsafe {
                super::push_button_connect_clicked(
                    widget.pin_mut(),
                    Self::on_click,
                    this.clone().into_raw().cast(),
                );
            }
            Self {
                widget: Widget::new(widget),
                on_click: Callback::new(),
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

    fn on_click(this: *const u8) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_click.signal(());
        }
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}
