use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::CheckButtonExt};

use crate::{
    AsWindow, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct CheckBox {
    on_click: Rc<Callback<()>>,
    widget: gtk4::CheckButton,
    handle: Widget,
}

impl CheckBox {
    pub(crate) fn new_impl(parent: impl AsWindow, radio: bool) -> Self {
        let widget = gtk4::CheckButton::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_click = Rc::new(Callback::new());
        widget.connect_toggled({
            let on_click = Rc::downgrade(&on_click);
            move |widget| {
                if let Some(on_click) = on_click.upgrade() {
                    if !radio || widget.is_active() {
                        on_click.signal(());
                    }
                }
            }
        });
        Self {
            on_click,
            widget,
            handle,
        }
    }

    pub fn new(parent: impl AsWindow) -> Self {
        Self::new_impl(parent, false)
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
        self.widget
            .label()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_label(Some(s.as_ref()));
        self.handle.reset_preferred_size();
    }

    pub fn is_checked(&self) -> bool {
        self.widget.is_active()
    }

    pub fn set_checked(&mut self, v: bool) {
        self.widget.set_active(v);
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

#[derive(Debug)]
pub struct RadioButton {
    handle: CheckBox,
}

impl RadioButton {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = CheckBox::new_impl(parent, true);
        Self { handle }
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
        self.handle.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(s);
    }

    pub fn is_checked(&self) -> bool {
        self.handle.is_checked()
    }

    pub fn set_checked(&mut self, v: bool) {
        self.handle.set_checked(v);
    }

    pub async fn wait_click(&self) {
        self.handle.wait_click().await
    }
}
