use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::NSProgressIndicator;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    view: Retained<NSProgressIndicator>,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

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

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn minimum(&self) -> usize {
        self.view.minValue() as _
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.view.setMinValue(v as _);
    }

    pub fn maximum(&self) -> usize {
        self.view.maxValue() as _
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.view.setMaxValue(v as _);
    }

    pub fn pos(&self) -> usize {
        self.view.doubleValue() as _
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.view.setDoubleValue(pos as _);
    }

    pub fn is_indeterminate(&self) -> bool {
        self.view.isIndeterminate()
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
