use inherit_methods_macro::inherit_methods;
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};

use crate::ui::{QtAlignmentFlag, Widget, impl_static_cast};

#[derive(Debug)]
pub struct Label {
    widget: Widget<ffi::QLabel>,
}

#[inherit_methods(from = "self.widget")]
impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_label(parent.as_window().as_qt()) };
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

    pub fn text(&self) -> String {
        self.widget.as_ref().text().into()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.pin_mut().setText(&s.as_ref().into());
    }

    pub fn halign(&self) -> HAlign {
        let flag = self.widget.as_ref().alignment();
        if flag.contains(QtAlignmentFlag::AlignRight) {
            HAlign::Right
        } else if flag.contains(QtAlignmentFlag::AlignHCenter) {
            HAlign::Center
        } else if flag.contains(QtAlignmentFlag::AlignJustify) {
            HAlign::Stretch
        } else {
            HAlign::Left
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        let mut flag = self.widget.as_ref().alignment() as i32;
        flag &= 0xFFF0;
        match align {
            HAlign::Left => flag |= QtAlignmentFlag::AlignLeft as i32,
            HAlign::Center => flag |= QtAlignmentFlag::AlignHCenter as i32,
            HAlign::Right => flag |= QtAlignmentFlag::AlignRight as i32,
            HAlign::Stretch => flag |= QtAlignmentFlag::AlignJustify as i32,
        }
        unsafe {
            self.widget
                .pin_mut()
                .setAlignment(std::mem::transmute::<i32, QtAlignmentFlag>(flag));
        }
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

        unsafe fn new_label(parent: *mut QWidget) -> UniquePtr<QLabel>;

        fn alignment(self: &QLabel) -> QtAlignmentFlag;
        fn setAlignment(self: Pin<&mut QLabel>, flag: QtAlignmentFlag);
        fn text(self: &QLabel) -> QString;
        fn setText(self: Pin<&mut QLabel>, s: &QString);
    }
}
