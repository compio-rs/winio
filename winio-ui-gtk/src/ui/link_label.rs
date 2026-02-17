use std::rc::Rc;

use compio_log::info;
use gtk4::{
    glib::{Propagation, object::Cast},
    prelude::ButtonExt,
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, ui::Widget};

#[derive(Debug)]
pub struct LinkLabel {
    on_click: Rc<Callback<()>>,
    widget: gtk4::LinkButton,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::LinkButton::new("");
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_click = Rc::new(Callback::new());
        widget.connect_activate_link({
            let on_click = on_click.clone();
            move |button| {
                if button.uri().is_empty() {
                    on_click.signal::<GlobalRuntime>(());
                    Propagation::Stop
                } else {
                    info!("LinkLabel will open URI {:?}", button.uri());
                    Propagation::Proceed
                }
            }
        });
        Ok(Self {
            on_click,
            widget,
            handle,
        })
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
        self.widget.set_label(s.as_ref());
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn uri(&self) -> Result<String> {
        Ok(self.widget.uri().to_string())
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.set_uri(s.as_ref());
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(LinkLabel, handle);
