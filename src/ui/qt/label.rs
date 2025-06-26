use crate::{
    AsRawWindow, AsWindow, HAlign, Point, Size,
    ui::{QtAlignmentFlag, Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Label {
    widget: Widget<ffi::QLabel>,
}

impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_label(parent.as_window().as_raw_window()) };
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { widget }
    }

    pub fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
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
        self.widget.as_ref().text().into()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.pin_mut().setText(&s.as_ref().into());
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

impl_static_cast!(
    ffi::QLabel,
    ffi::QWidget,
    ffi::static_cast_QLabel_QWidget,
    ffi::static_cast_mut_QLabel_QWidget
);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/label.hpp");

        type QWidget = crate::ui::QWidget;
        type QLabel;
        type QString = crate::ui::QString;
        type QtAlignmentFlag = crate::ui::QtAlignmentFlag;

        unsafe fn new_label(parent: *mut QWidget) -> UniquePtr<QLabel>;

        fn static_cast_QLabel_QWidget(w: &QLabel) -> &QWidget;
        fn static_cast_mut_QLabel_QWidget(w: Pin<&mut QLabel>) -> Pin<&mut QWidget>;

        fn text(self: &QLabel) -> QString;
        fn setText(self: Pin<&mut QLabel>, s: &QString);

        fn label_get_alignment(w: &QLabel) -> QtAlignmentFlag;
        fn label_set_alignment(w: Pin<&mut QLabel>, flag: QtAlignmentFlag);
    }
}
