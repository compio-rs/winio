use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Result,
    widgets::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Progress {
    widget: Widget<ffi::QProgressBar>,
    saved: (usize, usize, usize),
}

#[inherit_methods(from = "self.widget")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_progress_bar(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        widget.pin_mut().setRange(0, 10)?;
        widget.pin_mut().setValue(0)?;
        Ok(Self {
            widget,
            saved: (0, 10, 0),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.widget.as_ref().minimum()? as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?.max(v);
        let value = self.pos()?.clamp(v, max);
        self.widget.pin_mut().setRange(v as _, max as _)?;
        self.widget.pin_mut().setValue(value as _)?;
        if v != max {
            self.saved = (v, max, value);
        }
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.widget.as_ref().maximum()? as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?.min(v);
        let value = self.pos()?.clamp(min, v);
        self.widget.pin_mut().setRange(min as _, v as _)?;
        self.widget.pin_mut().setValue(value as _)?;
        if min != v {
            self.saved = (min, v, value);
        }
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        Ok(self.widget.as_ref().value()? as _)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        let min = self.minimum()?;
        let max = self.maximum()?;
        self.widget.pin_mut().setValue(pos.clamp(min, max) as _)?;
        Ok(())
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        Ok(self.minimum()? == 0 && self.maximum()? == 0)
    }

    pub fn set_indeterminate(&mut self, v: bool) -> Result<()> {
        if v {
            if !self.is_indeterminate()? {
                self.saved = (self.minimum()?, self.maximum()?, self.pos()?);
            }
            self.widget.pin_mut().setRange(0, 0)?;
            self.widget.pin_mut().setValue(0)?;
        } else if self.is_indeterminate()? {
            let (min, max, value) = self.saved;
            self.widget.pin_mut().setRange(min as _, max as _)?;
            self.widget.pin_mut().setValue(value as _)?;
        }
        Ok(())
    }
}

winio_handle::impl_as_widget!(Progress, widget);

impl_static_cast!(ffi::QProgressBar, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/progress.hpp");

        type QWidget = crate::widgets::QWidget;
        type QProgressBar;

        unsafe fn new_progress_bar(parent: *mut QWidget) -> Result<UniquePtr<QProgressBar>>;

        fn setRange(self: Pin<&mut QProgressBar>, min: i32, max: i32) -> Result<()>;

        fn minimum(self: &QProgressBar) -> Result<i32>;

        fn maximum(self: &QProgressBar) -> Result<i32>;

        fn setValue(self: Pin<&mut QProgressBar>, v: i32) -> Result<()>;
        fn value(self: &QProgressBar) -> Result<i32>;
    }
}
