use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct ComboBox {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl ComboBox {
    pub async fn wait_change(&self) {
        todo!()
    }

    pub async fn wait_select(&self) {
        todo!()
    }

    pub fn selection(&self) -> Option<usize> {
        todo!()
    }

    pub fn set_selection(&self, _i: Option<usize>) {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_editable(&self) -> bool {
        todo!()
    }

    pub fn set_editable(&self, _v: bool) {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    pub fn clear(&self) {
        todo!()
    }

    pub fn get(&self, _i: usize) -> String {
        todo!()
    }

    pub fn set<S>(&self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn insert<S>(&self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn remove(&self, _i: usize) {
        todo!()
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

impl_as_widget!(ComboBox, inner);
