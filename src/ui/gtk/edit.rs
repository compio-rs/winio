use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::EditableExt};

use crate::{
    AsWindow, HAlign, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct Edit {
    on_changed: Rc<Callback<()>>,
    widget: gtk4::Entry,
    handle: Widget,
}

impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::Entry::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_changed = Rc::new(Callback::new());
        widget.connect_changed({
            let on_changed = Rc::downgrade(&on_changed);
            move |_| {
                if let Some(on_changed) = on_changed.upgrade() {
                    on_changed.signal(());
                }
            }
        });
        Self {
            on_changed,
            widget,
            handle,
        }
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, s: Size) {
        self.handle.set_size(s);
    }

    pub fn text(&self) -> String {
        self.widget.text().to_string()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s.as_ref());
    }

    pub fn halign(&self) -> HAlign {
        let align = self.widget.alignment();
        if align == 0.0 {
            HAlign::Left
        } else if align == 1.0 {
            HAlign::Right
        } else {
            HAlign::Center
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        let align = match align {
            HAlign::Left => 0.0,
            HAlign::Right => 1.0,
            HAlign::Center => 0.5,
        };
        self.widget.set_alignment(align);
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}
