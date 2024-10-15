use crate::{
    AsRawWindow, AsWindow, HAlign, Point, Size,
    ui::{QtAlignmentFlag, Widget},
};

#[derive(Debug)]
pub struct Label {
    widget: Widget,
}

impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_label(parent.as_window().as_raw_window()) };
        widget.pin_mut().show();
        Self {
            widget: Widget::new(widget),
        }
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

    pub fn text(&self) -> String {
        ffi::label_get_text(self.widget.as_ref())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        ffi::label_set_text(self.widget.pin_mut(), s.as_ref())
    }

    pub fn halign(&self) -> HAlign {
        let flag = ffi::label_get_alignment(self.widget.as_ref());
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
        let mut flag = ffi::label_get_alignment(self.widget.as_ref()) as i32;
        flag &= 0xFFF0;
        match align {
            HAlign::Left => flag |= QtAlignmentFlag::AlignLeft as i32,
            HAlign::Center => flag |= QtAlignmentFlag::AlignHCenter as i32,
            HAlign::Right => flag |= QtAlignmentFlag::AlignRight as i32,
            HAlign::Stretch => flag |= QtAlignmentFlag::AlignJustify as i32,
        }
        unsafe {
            ffi::label_set_alignment(
                self.widget.pin_mut(),
                std::mem::transmute::<i32, QtAlignmentFlag>(flag),
            );
        }
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/label.hpp");

        type QWidget = crate::ui::QWidget;
        type QtAlignmentFlag = crate::ui::QtAlignmentFlag;

        unsafe fn new_label(parent: *mut QWidget) -> UniquePtr<QWidget>;

        fn label_get_text(w: &QWidget) -> String;
        fn label_set_text(w: Pin<&mut QWidget>, s: &str);

        fn label_get_alignment(w: &QWidget) -> QtAlignmentFlag;
        fn label_set_alignment(w: Pin<&mut QWidget>, flag: QtAlignmentFlag);
    }
}
