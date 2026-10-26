use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct UpDownEdit {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl UpDownEdit {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
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

    pub fn minimum(&self) -> Result<usize> {
        not_impl()
    }

    pub fn set_minimum(&mut self, _v: usize) -> Result<()> {
        not_impl()
    }

    pub fn maximum(&self) -> Result<usize> {
        not_impl()
    }

    pub fn set_maximum(&mut self, _v: usize) -> Result<()> {
        not_impl()
    }

    pub fn step(&self) -> Result<usize> {
        not_impl()
    }

    pub fn set_step(&mut self, _v: usize) -> Result<()> {
        not_impl()
    }

    pub fn value(&self) -> Result<usize> {
        not_impl()
    }

    pub fn set_value(&mut self, _v: usize) -> Result<()> {
        not_impl()
    }

    pub async fn wait_change(&self) {
        not_impl()
    }
}

winio_handle::impl_as_widget!(UpDownEdit, handle);
