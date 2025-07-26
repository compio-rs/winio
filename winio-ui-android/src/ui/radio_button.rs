use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct RadioButton {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl RadioButton {
    pub async fn wait_click(&self) {
        todo!()
    }

    pub fn text(&self) -> String;

    pub fn set_text<S>(&self, _text: S)
    where
        S: AsRef<str>;

    pub fn is_checked(&self) -> bool {
        todo!()
    }

    pub fn set_checked(&self, _v: bool) {
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

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

impl_as_widget!(RadioButton, inner);
