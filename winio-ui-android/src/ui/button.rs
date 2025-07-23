use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct Button {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Button {
    pub async fn wait_click(&self) {
        todo!()
    }

    pub fn text(&self) -> String;

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>;

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

impl_as_widget!(Button, inner);
