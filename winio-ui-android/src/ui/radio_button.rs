use {
    winio_handle::AsWindow,
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct RadioButton;

impl RadioButton {
    pub async fn wait_click(&self) {
        todo!()
    }

    pub fn text(&self) -> String {
        todo!()
    }

    pub fn set_text<S>(&mut self, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn is_checked(&self) -> bool {
        todo!()
    }

    pub fn set_checked(&self, _v: bool) {
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
