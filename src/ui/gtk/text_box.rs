use std::rc::Rc;

use gtk4::{
    Justification, WrapMode,
    glib::object::Cast,
    prelude::{TextBufferExt, TextViewExt},
};

use crate::{
    AsWindow, HAlign, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct TextBox {
    on_changed: Rc<Callback<()>>,
    widget: gtk4::TextView,
    handle: Widget,
}

impl TextBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let container = gtk4::ScrolledWindow::new();
        let widget = gtk4::TextView::new();
        container.set_child(Some(&widget));

        widget.set_wrap_mode(WrapMode::Char);
        let handle = Widget::new(parent, unsafe { container.clone().unsafe_cast() });

        let buffer = widget.buffer();
        let on_changed = Rc::new(Callback::new());
        buffer.connect_changed({
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
        let buffer = self.widget.buffer();
        buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.buffer().set_text(s.as_ref());
        self.handle.reset_preferred_size();
    }

    pub fn halign(&self) -> HAlign {
        let align = self.widget.justification();
        match align {
            Justification::Center => HAlign::Center,
            Justification::Right => HAlign::Right,
            Justification::Fill => HAlign::Stretch,
            _ => HAlign::Left,
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        let align = match align {
            HAlign::Left => Justification::Left,
            HAlign::Center => Justification::Center,
            HAlign::Right => Justification::Right,
            HAlign::Stretch => Justification::Fill,
        };
        self.widget.set_justification(align);
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}
