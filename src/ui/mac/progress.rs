use objc2::rc::Id;
use objc2_app_kit::NSProgressIndicator;
use objc2_foundation::MainThreadMarker;

use crate::{AsRawWindow, AsWindow, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    view: Id<NSProgressIndicator>,
}

impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSProgressIndicator::new(mtm);
            view.setIndeterminate(false);
            view.setUsesThreadedAnimation(false);
            let handle =
                Widget::from_nsview(parent.as_window().as_raw_window(), Id::cast(view.clone()));

            Self { handle, view }
        }
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn range(&self) -> (usize, usize) {
        unsafe { (self.view.minValue() as _, self.view.maxValue() as _) }
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        unsafe {
            self.view.setMinValue(min as _);
            self.view.setMaxValue(max as _);
        }
    }

    pub fn pos(&self) -> usize {
        unsafe { self.view.doubleValue() as _ }
    }

    pub fn set_pos(&mut self, pos: usize) {
        unsafe {
            self.view.setDoubleValue(pos as _);
        }
    }

    pub fn is_indeterminate(&self) -> bool {
        unsafe { self.view.isIndeterminate() }
    }

    pub fn set_indeterminate(&mut self, v: bool) {
        unsafe {
            self.view.setIndeterminate(v);
        }
    }
}
