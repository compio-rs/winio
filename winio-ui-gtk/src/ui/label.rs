use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{Result, ui::Widget};

#[derive(Debug)]
pub struct Label {
    widget: gtk4::Label,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::Label::new(None);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        Ok(Self { widget, handle })
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
        Ok(self.widget.text().to_string())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.set_text(s.as_ref());
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        let align = self.widget.xalign();
        let align = if align == 0.0 {
            HAlign::Left
        } else if align == 1.0 {
            HAlign::Right
        } else {
            HAlign::Center
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let align = match align {
            HAlign::Left => 0.0,
            HAlign::Right => 1.0,
            _ => 0.5,
        };
        self.widget.set_xalign(align);
        Ok(())
    }
}

winio_handle::impl_as_widget!(Label, handle);
