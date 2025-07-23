use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Orient, Point, Size},
};

#[derive(Debug)]
pub struct Slider {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Slider {
    pub async fn wait_change(&self) {
        todo!()
    }

    pub fn orient(&self) -> Orient {
        todo!()
    }

    pub fn set_orient(&mut self, _v: Orient) {
        todo!()
    }

    pub fn minimum(&self) -> usize {
        todo!()
    }

    pub fn set_minimum(&mut self, _v: usize) {
        todo!()
    }

    pub fn maximum(&self) -> usize {
        todo!()
    }

    pub fn set_maximum(&mut self, _v: usize) {
        todo!()
    }

    pub fn freq(&self) -> usize {
        todo!()
    }

    pub fn set_freq(&mut self, _v: usize) {
        todo!()
    }

    pub fn pos(&self) -> usize {
        todo!()
    }

    pub fn set_pos(&mut self, _pos: usize) {
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

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

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

impl_as_widget!(Slider, inner);
