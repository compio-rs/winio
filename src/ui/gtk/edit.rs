use std::{
    io,
    rc::{Rc, Weak},
};

use gtk4::{glib::object::Cast, prelude::EditableExt};

use super::callback::Callback;
use crate::{AsContainer, Point, Size, Widget};

pub struct Edit {
    widget: gtk4::Entry,
    handle: Rc<Widget>,
    on_changed: Callback<()>,
}

impl Edit {
    pub fn new(parent: impl AsContainer) -> io::Result<Rc<Self>> {
        let widget = gtk4::Entry::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Ok(Rc::new_cyclic(|this: &Weak<Self>| {
            widget.connect_changed({
                let this = this.clone();
                move |_| {
                    if let Some(this) = this.upgrade() {
                        this.on_changed.signal(());
                    }
                }
            });
            Self {
                widget,
                handle,
                on_changed: Callback::new(),
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
        Ok(self.widget.text().to_string())
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.widget.set_text(s.as_ref());
        Ok(())
    }

    pub async fn wait_changed(&self) {
        self.on_changed.wait().await
    }
}
