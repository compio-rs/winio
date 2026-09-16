use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Image, Result, widgets::Widget};

#[derive(Debug)]
pub struct Picture {
    image: gtk4::Picture,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Picture {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let image = gtk4::Picture::new();
        let handle = Widget::new(parent, unsafe { image.clone().unsafe_cast() })?;
        Ok(Self { image, handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_image(&mut self, image: Option<&Image>) -> Result<()> {
        match image {
            Some(image) => self.image.set_paintable(Some(image.texture())),
            None => self.image.set_paintable(gtk4::gdk::Paintable::NONE),
        }
        self.handle.reset_preferred_size();
        Ok(())
    }
}

winio_handle::impl_as_widget!(Picture, handle);
