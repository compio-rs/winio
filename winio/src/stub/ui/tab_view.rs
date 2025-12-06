use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        not_impl()
    }

    pub fn set_selection(&mut self, _i: usize) -> Result<()> {
        not_impl()
    }

    pub fn insert(&mut self, _i: usize, _item: &TabViewItem) -> Result<()> {
        not_impl()
    }

    pub fn remove(&mut self, _i: usize) -> Result<()> {
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

    pub async fn wait_select(&self) {
        not_impl()
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
pub struct TabViewItem {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl TabViewItem {
    pub fn new() -> Result<Self> {
        not_impl()
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn size(&self) -> Result<Size>;
}

winio_handle::impl_as_container!(TabViewItem, handle);
