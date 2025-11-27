use std::rc::Rc;

use gtk4::{
    glib::object::Cast,
    prelude::{CheckButtonExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, ui::Widget};

#[derive(Debug)]
pub struct CheckBox {
    on_click: Rc<Callback<()>>,
    widget: gtk4::CheckButton,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl CheckBox {
    pub(crate) fn new_impl(parent: impl AsContainer, radio: bool) -> Result<Self> {
        let widget = gtk4::CheckButton::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_click = Rc::new(Callback::new());
        widget.connect_toggled({
            let on_click = on_click.clone();
            move |widget| {
                if !radio || widget.is_active() {
                    on_click.signal::<GlobalRuntime>(());
                }
            }
        });
        Ok(Self {
            on_click,
            widget,
            handle,
        })
    }

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Self::new_impl(parent, false)
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(self
            .widget
            .label()
            .map(|s| s.to_string())
            .unwrap_or_default())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.set_label(Some(s.as_ref()));
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn is_checked(&self) -> Result<bool> {
        Ok(self.widget.is_active())
    }

    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        self.widget.set_active(v);
        Ok(())
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CheckBox::new_impl(parent, true)?;
        let hidden = gtk4::CheckButton::new();
        hidden.set_visible(false);
        handle.widget.set_group(Some(&hidden));
        Ok(Self { handle, hidden })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        if v {
            self.handle.set_checked(true)?;
        } else {
            self.hidden.set_active(true);
        }
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.handle.wait_click().await
    }
}

winio_handle::impl_as_widget!(RadioButton, handle);
