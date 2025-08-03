use std::rc::Rc;

use gtk4::{
    Justification, WrapMode,
    glib::object::Cast,
    prelude::{TextBufferExt, TextViewExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
pub struct TextBox {
    on_changed: Rc<Callback<()>>,
    widget: gtk4::TextView,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
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
            let on_changed = on_changed.clone();
            move |_| {
                on_changed.signal::<GlobalRuntime>(());
            }
        });
        Self {
            on_changed,
            widget,
            handle,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size {
        let size = self.preferred_size();
        Size::new(size.width, 0.0)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

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

winio_handle::impl_as_widget!(TextBox, handle);
