use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Label {
    widget: gtk4::Label,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::Label::new(None);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Self { widget, handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn text(&self) -> String {
        self.widget.text().to_string()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s.as_ref());
        self.handle.reset_preferred_size();
    }

    pub fn halign(&self) -> HAlign {
        let align = self.widget.xalign();
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
        self.widget.set_xalign(align);
    }
}

winio_handle::impl_as_widget!(Label, handle);
