use {
    winio_handle::{AsWindow, RawWidget, impl_as_widget},
    winio_primitive::{HAlign, Point, Size},
};

#[derive(Debug)]
pub struct Label {
    inner: RawWidget,
}

//noinspection SpellCheckingInspection
impl Label {
    pub fn text(&self) -> String {
        todo!()
    }

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn halign(&self) -> HAlign {
        todo!()
    }

    pub fn set_halign(&mut self, _align: HAlign) {
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn set_visible(&mut self, _v: bool) {
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

    pub fn set_size(&mut self, _size: Size) {
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

impl_as_widget!(Label, inner);
