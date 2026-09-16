use std::rc::Rc;

use gtk4::{
    glib::object::Cast,
    prelude::{BoxExt, ButtonExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Image, Result, widgets::Widget};

#[derive(Debug)]
pub struct Button {
    on_click: Rc<Callback<()>>,
    image: gtk4::Picture,
    label: gtk4::Label,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::Button::new();
        let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        content.set_halign(gtk4::Align::Center);
        content.set_valign(gtk4::Align::Center);
        content.set_hexpand(false);
        content.set_vexpand(false);
        let image = gtk4::Picture::new();
        image.set_visible(false);
        let label = gtk4::Label::new(None);
        label.set_visible(false);
        content.append(&image);
        content.append(&label);
        widget.set_child(Some(&content));
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_click = Rc::new(Callback::new());
        widget.connect_clicked({
            let on_click = on_click.clone();
            move |_| {
                on_click.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_click,
            image,
            label,
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
        Ok(self.label.text().to_string())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        self.label.set_text(s);
        self.label.set_visible(!s.is_empty());
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => {
                self.image.set_paintable(Some(icon.texture()));
                self.image.set_visible(true);
            }
            None => {
                self.image.set_paintable(gtk4::gdk::Paintable::NONE);
                self.image.set_visible(false);
            }
        }
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(Button, handle);
