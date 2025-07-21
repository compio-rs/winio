use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::ButtonExt};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
pub struct Button {
    on_click: Rc<Callback<()>>,
    widget: gtk4::Button,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::Button::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_click = Rc::new(Callback::new());
        widget.connect_clicked({
            let on_click = on_click.clone();
            move |_| {
                on_click.signal::<GlobalRuntime>(());
            }
        });
        Self {
            on_click,
            widget,
            handle,
        }
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
        self.widget
            .label()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_label(s.as_ref());
        self.handle.reset_preferred_size();
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(Button, handle);
