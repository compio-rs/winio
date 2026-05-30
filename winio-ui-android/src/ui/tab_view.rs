use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{AView, BaseWidget, Result};

#[derive(Debug)]
pub struct TabView {
    handle: BaseWidget<AView<'static>>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        todo!()
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
        todo!()
    }

    pub fn set_selection(&mut self, _i: usize) -> Result<()> {
        todo!()
    }

    pub fn insert(&mut self, _i: usize, _item: &TabViewItem) -> Result<()> {
        todo!()
    }

    pub fn remove(&mut self, _i: usize) -> Result<()> {
        todo!()
    }

    pub fn len(&self) -> Result<usize> {
        todo!()
    }

    pub fn is_empty(&self) -> Result<bool> {
        todo!()
    }

    pub fn clear(&mut self) -> Result<()> {
        todo!()
    }

    pub async fn wait_select(&self) {
        todo!()
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
pub struct TabViewItem {
    handle: BaseWidget<AView<'static>>,
}

#[inherit_methods(from = "self.handle")]
impl TabViewItem {
    pub fn new() -> Result<Self> {
        todo!()
    }

    pub fn text(&self) -> Result<String> {
        todo!()
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        todo!()
    }

    pub fn size(&self) -> Result<Size>;
}

winio_handle::impl_as_container!(TabViewItem, handle);
