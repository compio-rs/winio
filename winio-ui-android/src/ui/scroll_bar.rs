use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Orient, Point, Size};

use crate::{AView, BaseWidget, Result};

#[derive(Debug)]
pub struct ScrollBar {
    inner: BaseWidget<AView<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl ScrollBar {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        todo!()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn orient(&self) -> Result<Orient> {
        todo!()
    }

    pub fn set_orient(&mut self, _v: Orient) -> Result<()> {
        todo!()
    }

    pub fn minimum(&self) -> Result<usize> {
        todo!()
    }

    pub fn set_minimum(&mut self, _v: usize) -> Result<()> {
        todo!()
    }

    pub fn maximum(&self) -> Result<usize> {
        todo!()
    }

    pub fn set_maximum(&mut self, _v: usize) -> Result<()> {
        todo!()
    }

    pub fn page(&self) -> Result<usize> {
        todo!()
    }

    pub fn set_page(&mut self, _v: usize) -> Result<()> {
        todo!()
    }

    pub fn pos(&self) -> Result<usize> {
        todo!()
    }

    pub fn set_pos(&mut self, _pos: usize) -> Result<()> {
        todo!()
    }

    pub async fn wait_change(&self) {
        todo!()
    }
}

impl_as_widget!(ScrollBar, inner);
