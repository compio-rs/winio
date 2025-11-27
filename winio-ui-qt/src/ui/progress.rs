use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Result,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Progress {
    widget: Widget<ffi::QProgressBar>,
}

#[inherit_methods(from = "self.widget")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_progress_bar(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { widget })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

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
        self.widget.pin_mut().setMinimum(v as _)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.widget.as_ref().maximum()? as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        self.widget.pin_mut().setMaximum(v as _)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        Ok(self.widget.as_ref().value()? as _)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        self.widget.pin_mut().setValue(pos as _)?;
        Ok(())
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        Ok(self.minimum()? == 0 && self.maximum()? == 0)
    }

    pub fn set_indeterminate(&mut self, v: bool) -> Result<()> {
        if v {
            self.widget.pin_mut().setRange(0, 0)?;
        } else {
            self.widget.pin_mut().setRange(0, 1)?;
        }
        Ok(())
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

        unsafe fn new_progress_bar(parent: *mut QWidget) -> Result<UniquePtr<QProgressBar>>;

        fn setRange(self: Pin<&mut QProgressBar>, min: i32, max: i32) -> Result<()>;

        fn minimum(self: &QProgressBar) -> Result<i32>;
        fn setMinimum(self: Pin<&mut QProgressBar>, v: i32) -> Result<()>;

        fn maximum(self: &QProgressBar) -> Result<i32>;
        fn setMaximum(self: Pin<&mut QProgressBar>, v: i32) -> Result<()>;

        fn setValue(self: Pin<&mut QProgressBar>, v: i32) -> Result<()>;
        fn value(self: &QProgressBar) -> Result<i32>;
    }
}
