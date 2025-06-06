use crate::{AsRawWindow, AsWindow, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Progress {
    widget: Widget,
}

impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_progress_bar(parent.as_window().as_raw_window()) };
        widget.pin_mut().setVisible(true);
        Self {
            widget: Widget::new(widget),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.widget.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.widget.size()
    }

    pub fn set_size(&mut self, s: Size) {
        self.widget.set_size(s);
    }

    pub fn range(&self) -> (usize, usize) {
        (
            ffi::progress_bar_get_minimum(self.widget.as_ref()) as _,
            ffi::progress_bar_get_maximum(self.widget.as_ref()) as _,
        )
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        ffi::progress_bar_set_range(self.widget.pin_mut(), min as _, max as _);
    }

    pub fn pos(&self) -> usize {
        ffi::progress_bar_get_value(self.widget.as_ref()) as _
    }

    pub fn set_pos(&mut self, pos: usize) {
        ffi::progress_bar_set_value(self.widget.pin_mut(), pos as _);
    }

    pub fn is_indeterminate(&self) -> bool {
        let (min, max) = self.range();
        min == 0 && max == 0
    }

    pub fn set_indeterminate(&mut self, v: bool) {
        if v {
            self.set_range(0, 0);
        } else {
            self.set_range(0, 1);
        }
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/progress.hpp");

        type QWidget = crate::ui::QWidget;

        unsafe fn new_progress_bar(parent: *mut QWidget) -> UniquePtr<QWidget>;

        fn progress_bar_set_range(w: Pin<&mut QWidget>, min: i32, max: i32);
        fn progress_bar_get_minimum(w: &QWidget) -> i32;
        fn progress_bar_get_maximum(w: &QWidget) -> i32;

        fn progress_bar_set_value(w: Pin<&mut QWidget>, v: i32);
        fn progress_bar_get_value(w: &QWidget) -> i32;
    }
}
