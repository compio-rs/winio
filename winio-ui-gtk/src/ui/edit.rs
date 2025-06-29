use std::rc::Rc;

use gtk4::{
    glib::object::Cast,
    prelude::{EditableExt, EntryExt},
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRuntime, ui::Widget};

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
                    on_changed.signal::<GlobalRuntime>(());
                }
            }
        });
        Self {
            on_changed,
            widget,
            handle,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
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
        self.handle.reset_preferred_size();
    }

    pub fn is_password(&self) -> bool {
        !self.widget.is_visible()
    }

    pub fn set_password(&mut self, v: bool) {
        self.widget.set_input_purpose(if v {
            gtk4::InputPurpose::Password
        } else {
            gtk4::InputPurpose::FreeForm
        });
        self.widget.set_visibility(!v);
    }

    pub fn halign(&self) -> HAlign {
        let align = EditableExt::alignment(&self.widget);
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
            _ => 0.5,
        };
        EditableExt::set_alignment(&self.widget, align);
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}
