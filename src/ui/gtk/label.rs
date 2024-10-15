use gtk4::glib::object::Cast;

use crate::{AsWindow, HAlign, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Label {
    widget: gtk4::Label,
    handle: Widget,
}

impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::Label::new(None);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Self { widget, handle }
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
