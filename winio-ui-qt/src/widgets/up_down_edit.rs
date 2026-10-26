use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result,
    widgets::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct UpDownEdit {
    on_change: Box<Callback>,
    widget: Widget<ffi::QSpinBox>,
}

#[inherit_methods(from = "self.widget")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_spin_box(parent.as_container().as_qt()) }?;
        let on_change = Box::new(Callback::new());
        unsafe {
            ffi::spin_box_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_change.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { on_change, widget })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

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

    pub fn step(&self) -> Result<usize> {
        Ok(self.widget.as_ref().singleStep()? as _)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        self.widget.pin_mut().setSingleStep(v as _)?;
        Ok(())
    }

    pub fn value(&self) -> Result<usize> {
        Ok(self.widget.as_ref().value()? as _)
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        self.widget.pin_mut().setValue(v as _)?;
        Ok(())
    }

    fn on_changed(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

impl_static_cast!(ffi::QSpinBox, ffi::QWidget);

winio_handle::impl_as_widget!(UpDownEdit, widget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/up_down_edit.hpp");

        type QWidget = crate::widgets::QWidget;
        type QSpinBox;
        type QString = crate::common::QString;

        unsafe fn new_spin_box(parent: *mut QWidget) -> Result<UniquePtr<QSpinBox>>;
        unsafe fn spin_box_connect_changed(
            w: Pin<&mut QSpinBox>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn minimum(self: &QSpinBox) -> Result<i32>;
        fn setMinimum(self: Pin<&mut QSpinBox>, v: i32) -> Result<()>;

        fn maximum(self: &QSpinBox) -> Result<i32>;
        fn setMaximum(self: Pin<&mut QSpinBox>, v: i32) -> Result<()>;

        fn singleStep(self: &QSpinBox) -> Result<i32>;
        fn setSingleStep(self: Pin<&mut QSpinBox>, v: i32) -> Result<()>;

        fn value(self: &QSpinBox) -> Result<i32>;
        fn setValue(self: Pin<&mut QSpinBox>, v: i32) -> Result<()>;
    }
}
