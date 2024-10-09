use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::ButtonExt};

use crate::{
    AsWindow, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct Button {
    on_click: Rc<Callback<()>>,
    widget: gtk4::Button,
    handle: Widget,
}

impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::Button::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_click = Rc::new(Callback::new());
        widget.connect_clicked({
            let on_click = Rc::downgrade(&on_click);
            move |_| {
                if let Some(on_click) = on_click.upgrade() {
                    on_click.signal(());
                }
            }
        });
        Self {
            on_click,
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
        self.widget
            .label()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_label(s.as_ref());
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}
