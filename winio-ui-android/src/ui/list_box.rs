use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct ListBox {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl ListBox {
    pub async fn wait_select(&self) {
        todo!()
    }

    pub fn is_selected(&self, _i: usize) -> bool {
        todo!()
    }

    pub fn set_selected(&mut self, _i: usize, _v: bool) {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn get(&self, _i: usize) -> String {
        todo!()
    }

    pub fn set<S>(&mut self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn insert<S>(&mut self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn remove(&mut self, _i: usize) {
        todo!()
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, visible: bool);

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn min_size(&self) -> Size {
        todo!()
    }

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

impl_as_widget!(ListBox, inner);
