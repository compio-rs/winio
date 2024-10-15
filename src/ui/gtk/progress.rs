use gtk4::glib::object::Cast;

use crate::{AsWindow, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Progress {
    widget: gtk4::ProgressBar,
    handle: Widget,
    min: usize,
    max: usize,
}

impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::ProgressBar::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Self {
            widget,
            handle,
            min: 0,
            max: 1,
        }
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, mut s: Size) {
        s.height = self.preferred_size().height;
        self.handle.set_size(s);
    }

    pub fn range(&self) -> (usize, usize) {
        (self.min, self.max)
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        let pos = self.pos();
        self.min = min;
        self.max = max;
        self.set_pos(pos);
    }

    pub fn pos(&self) -> usize {
        (self.widget.fraction() * ((self.max - self.min) as f64) + self.min as f64) as usize
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.widget
            .set_fraction(((pos - self.min) as f64) / ((self.max - self.min) as f64));
    }

    pub fn is_indeterminate(&self) -> bool {
        false
    }

    pub fn set_indeterminate(&mut self, _v: bool) {}
}
