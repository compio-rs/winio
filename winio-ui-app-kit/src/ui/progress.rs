use inherit_methods_macro::inherit_methods;
use objc2::rc::Retained;
use objc2_app_kit::NSProgressIndicator;
use objc2_foundation::MainThreadMarker;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    view: Retained<NSProgressIndicator>,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSProgressIndicator::new(mtm);
            view.setIndeterminate(false);
            view.setUsesThreadedAnimation(false);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            Self { handle, view }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        Size::new(0.0, 5.0)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

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
            if v {
                self.view.startAnimation(None);
            } else {
                self.view.stopAnimation(None);
            }
        }
    }
}

winio_handle::impl_as_widget!(Progress, handle);
