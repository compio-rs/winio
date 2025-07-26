use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct Progress {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Progress {
    pub fn range(&self) -> (usize, usize) {
        todo!()
    }

    pub fn set_range(&self, _min: usize, _max: usize) {
        todo!()
    }

    pub fn pos(&self) -> usize {
        todo!()
    }

    pub fn set_pos(&self, _pos: usize) {
        todo!()
    }

    pub fn is_indeterminate(&self) -> bool {
        todo!()
    }

    pub fn set_indeterminate(&self, _v: bool) {
        todo!()
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&self, enabled: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn minimum(&self) -> usize {
        todo!()
    }

    pub fn set_minimum(&self, _v: usize) {
        todo!()
    }

    pub fn maximum(&self) -> usize {
        todo!()
    }

    pub fn set_maximum(&self, _v: usize) {
        todo!()
    }

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

impl_as_widget!(Progress, inner);
