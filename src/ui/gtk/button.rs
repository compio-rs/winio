use std::{
    io,
    rc::{Rc, Weak},
};

use gtk4::{glib::object::Cast, prelude::ButtonExt};

use super::callback::Callback;
use crate::{AsContainer, Point, Size, Widget};

pub struct Button {
    widget: gtk4::Button,
    handle: Rc<Widget>,
    on_click: Callback<()>,
}

impl Button {
    pub fn new(parent: impl AsContainer) -> io::Result<Rc<Self>> {
        let widget = gtk4::Button::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Ok(Rc::new_cyclic(|this: &Weak<Self>| {
            widget.connect_clicked({
                let this = this.clone();
                move |_| {
                    if let Some(this) = this.upgrade() {
                        this.on_click.signal(());
                    }
                }
            });
            Self {
                widget,
                handle,
                on_click: Callback::new(),
            }
        }))
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(self.handle.loc())
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p);
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(self.handle.size())
    }

    pub fn set_size(&self, s: Size) -> io::Result<()> {
        self.handle.set_size(s);
        Ok(())
    }

    pub fn text(&self) -> io::Result<String> {
        Ok(self
            .widget
            .label()
            .map(|s| s.to_string())
            .unwrap_or_default())
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.widget.set_label(s.as_ref());
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}
