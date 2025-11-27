use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{
    Result,
    ui::{QtAlignmentFlag, Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Label {
    widget: Widget<ffi::QLabel>,
}

#[inherit_methods(from = "self.widget")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_label(parent.as_container().as_qt()) }?;
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

    pub fn text(&self) -> Result<String> {
        Ok(self.widget.as_ref().text()?.try_into()?)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.pin_mut().setText(&s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        let flag = self.widget.as_ref().alignment()?;
        let align = if flag.contains(QtAlignmentFlag::AlignRight) {
            HAlign::Right
        } else if flag.contains(QtAlignmentFlag::AlignHCenter) {
            HAlign::Center
        } else if flag.contains(QtAlignmentFlag::AlignJustify) {
            HAlign::Stretch
        } else {
            HAlign::Left
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let mut flag = self.widget.as_ref().alignment()?;
        flag &= QtAlignmentFlag::from_bits_retain(0xFFF0);
        match align {
            HAlign::Left => flag |= QtAlignmentFlag::AlignLeft,
            HAlign::Center => flag |= QtAlignmentFlag::AlignHCenter,
            HAlign::Right => flag |= QtAlignmentFlag::AlignRight,
            HAlign::Stretch => flag |= QtAlignmentFlag::AlignJustify,
        }
        self.widget.pin_mut().setAlignment(flag)?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(Label, widget);

impl_static_cast!(ffi::QLabel, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/label.hpp");

        type QWidget = crate::ui::QWidget;
        type QLabel;
        type QString = crate::ui::QString;
        type QtAlignmentFlag = crate::ui::QtAlignmentFlag;

        unsafe fn new_label(parent: *mut QWidget) -> Result<UniquePtr<QLabel>>;

        fn alignment(self: &QLabel) -> Result<QtAlignmentFlag>;
        fn setAlignment(self: Pin<&mut QLabel>, flag: QtAlignmentFlag) -> Result<()>;
        fn text(self: &QLabel) -> Result<QString>;
        fn setText(self: Pin<&mut QLabel>, s: &QString) -> Result<()>;
    }
}
