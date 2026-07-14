use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct ListBox {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_multiple(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_multiple(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn is_selected(&self, _i: usize) -> Result<bool> {
        not_impl()
    }

    pub fn set_selected(&mut self, _i: usize, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn len(&self) -> Result<usize> {
        not_impl()
    }

    pub fn is_empty(&self) -> Result<bool> {
        not_impl()
    }

    pub fn clear(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn get(&self, _i: usize) -> Result<String> {
        not_impl()
    }

    pub fn set(&mut self, _i: usize, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn insert(&mut self, _i: usize, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn remove(&mut self, _i: usize) -> Result<()> {
        not_impl()
    }

    pub async fn wait_select(&self) {
        not_impl()
    }
}

winio_handle::impl_as_widget!(ListBox, handle);
