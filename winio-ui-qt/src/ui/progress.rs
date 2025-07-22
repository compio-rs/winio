use inherit_methods_macro::inherit_methods;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::ui::{Widget, impl_static_cast};

#[derive(Debug)]
pub struct Progress {
    widget: Widget<ffi::QProgressBar>,
}

#[inherit_methods(from = "self.widget")]
impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_progress_bar(parent.as_window().as_qt()) };
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { widget }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn minimum(&self) -> usize {
        self.widget.as_ref().minimum() as _
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.widget.pin_mut().setMinimum(v as _);
    }

    pub fn maximum(&self) -> usize {
        self.widget.as_ref().maximum() as _
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.widget.pin_mut().setMaximum(v as _);
    }

    pub fn pos(&self) -> usize {
        self.widget.as_ref().value() as _
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.widget.pin_mut().setValue(pos as _);
    }

    pub fn is_indeterminate(&self) -> bool {
        self.minimum() == 0 && self.maximum() == 0
    }

    pub fn set_indeterminate(&mut self, v: bool) {
        if v {
            self.widget.pin_mut().setRange(0, 0);
        } else {
            self.widget.pin_mut().setRange(0, 1);
        }
    }
}

winio_handle::impl_as_widget!(Progress, widget);

impl_static_cast!(ffi::QProgressBar, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/progress.hpp");

        type QWidget = crate::ui::QWidget;
        type QProgressBar;

        unsafe fn new_progress_bar(parent: *mut QWidget) -> UniquePtr<QProgressBar>;

        fn setRange(self: Pin<&mut QProgressBar>, min: i32, max: i32);

        fn minimum(self: &QProgressBar) -> i32;
        fn setMinimum(self: Pin<&mut QProgressBar>, v: i32);

        fn maximum(self: &QProgressBar) -> i32;
        fn setMaximum(self: Pin<&mut QProgressBar>, v: i32);

        fn setValue(self: Pin<&mut QProgressBar>, v: i32);
        fn value(self: &QProgressBar) -> i32;
    }
}
