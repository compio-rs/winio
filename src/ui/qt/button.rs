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

    pub fn text(&self) -> io::Result<String> {
        Ok(self.widget.text())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> io::Result<()> {
        self.widget.set_text(s.as_ref());
        Ok(())
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
