use crate::BaseWidget;
use {
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{HAlign, Point, Size},
};

#[derive(Debug)]
pub struct TextBox {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl TextBox {
    pub async fn wait_change(&self) {
        todo!()
    }

    pub fn text(&self) -> String;

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>;

    //noinspection SpellCheckingInspection
    pub fn halign(&self) -> HAlign {
        todo!()
    }

    //noinspection SpellCheckingInspection
    pub fn set_halign(&mut self, _align: HAlign) {
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

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

impl_as_widget!(TextBox, inner);
