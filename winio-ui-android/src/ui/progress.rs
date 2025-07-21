use {
    winio_handle::{AsWindow, RawWidget, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct Progress {
    inner: RawWidget,
}

impl Progress {
    pub fn range(&self) -> (usize, usize) {
        todo!()
    }

    pub fn set_range(&mut self, _min: usize, _max: usize) {
        todo!()
    }

    pub fn pos(&self) -> usize {
        todo!()
    }

    pub fn set_pos(&mut self, _pos: usize) {
        todo!()
    }

    pub fn is_indeterminate(&self) -> bool {
        todo!()
    }

    pub fn set_indeterminate(&mut self, _v: bool) {
        todo!()
    }

    pub fn set_visible(&mut self, _v: bool) {
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size {
        todo!()
    }

    pub fn set_size(&mut self, _v: Size) {
        todo!()
    }

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

impl_as_widget!(Progress, inner);
