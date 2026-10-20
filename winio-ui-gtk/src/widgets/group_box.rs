use gtk4::prelude::*;
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{Result, Widget};

#[derive(Debug)]
pub struct GroupBox {
    frame: gtk4::Frame,
    fixed: gtk4::Fixed,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let fixed = gtk4::Fixed::new();
        let frame = gtk4::Frame::new(None);
        frame.set_child(Some(&fixed));
        let handle = Widget::new(parent, unsafe { frame.clone().unsafe_cast() })?;
        Ok(Self {
            frame,
            fixed,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn client_size(&self) -> Result<Size> {
        let (mut width, mut height) = self.frame.size_request();
        if width < 0 {
            width = self.frame.width();
        }
        if height < 0 {
            height = self.frame.height();
        }
        if width <= 0 || height <= 0 {
            return Ok(Size::zero());
        }
        let label_height = self
            .frame
            .label_widget()
            .map(|label| {
                let label_width = label
                    .measure(gtk4::Orientation::Horizontal, -1)
                    .1
                    .min(width);
                label.measure(gtk4::Orientation::Vertical, label_width).0
            })
            .unwrap_or(0);
        Ok(Size::new(width as _, (height - label_height).max(0) as _))
    }

    pub fn text(&self) -> Result<String> {
        Ok(self
            .frame
            .label()
            .map(|s| s.to_string())
            .unwrap_or_default())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        self.frame
            .set_label(if s.is_empty() { None } else { Some(s) });
        self.handle.reset_preferred_size();
        Ok(())
    }
}

winio_handle::impl_as_widget!(GroupBox, handle);

impl AsContainer for GroupBox {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::gtk(&self.fixed)
    }
}
