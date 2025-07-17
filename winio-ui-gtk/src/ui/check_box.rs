use std::rc::Rc;

use gtk4::{
    glib::object::Cast,
    prelude::{CheckButtonExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
pub struct CheckBox {
    on_click: Rc<Callback<()>>,
    widget: gtk4::CheckButton,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
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
                        on_click.signal::<GlobalRuntime>(());
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

winio_handle::impl_as_widget!(CheckBox, handle);

#[derive(Debug)]
pub struct RadioButton {
    handle: CheckBox,
    hidden: gtk4::CheckButton,
}

#[inherit_methods(from = "self.handle")]
impl RadioButton {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = CheckBox::new_impl(parent.as_window(), true);
        let hidden = gtk4::CheckButton::new();
        hidden.set_visible(false);
        handle.widget.set_group(Some(&hidden));
        Self { handle, hidden }
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

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn is_checked(&self) -> bool;

    pub fn set_checked(&mut self, v: bool) {
        if v {
            self.handle.set_checked(true);
        } else {
            self.hidden.set_active(true);
        }
    }

    pub async fn wait_click(&self) {
        self.handle.wait_click().await
    }
}

winio_handle::impl_as_widget!(RadioButton, handle);
