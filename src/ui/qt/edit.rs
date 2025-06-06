use cxx::{ExternType, type_id};

use crate::{
    AsRawWindow, AsWindow, HAlign, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct Edit {
    on_changed: Box<Callback>,
    widget: Widget,
}

impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_line_edit(parent.as_window().as_raw_window()) };
        widget.pin_mut().setVisible(true);
        let on_changed = Box::new(Callback::new());
        unsafe {
            ffi::line_edit_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            );
        }
        Self {
            on_changed,
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

    pub fn text(&self) -> String {
        ffi::line_edit_get_text(self.widget.as_ref())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        ffi::line_edit_set_text(self.widget.pin_mut(), s.as_ref())
    }

    pub fn is_password(&self) -> bool {
        ffi::line_edit_is_password(self.widget.as_ref())
    }

    pub fn set_password(&mut self, v: bool) {
        ffi::line_edit_set_password(self.widget.pin_mut(), v);
    }

    pub fn halign(&self) -> HAlign {
        let flag = ffi::line_edit_get_alignment(self.widget.as_ref());
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
        let mut flag = ffi::line_edit_get_alignment(self.widget.as_ref()) as i32;
        flag &= 0xFFF0;
        match align {
            HAlign::Left => flag |= QtAlignmentFlag::AlignLeft as i32,
            HAlign::Center => flag |= QtAlignmentFlag::AlignHCenter as i32,
            HAlign::Right => flag |= QtAlignmentFlag::AlignRight as i32,
            HAlign::Stretch => flag |= QtAlignmentFlag::AlignJustify as i32,
        }
        unsafe {
            ffi::line_edit_set_alignment(
                self.widget.pin_mut(),
                std::mem::transmute::<i32, QtAlignmentFlag>(flag),
            );
        }
    }

    fn on_changed(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(());
        }
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum QtAlignmentFlag {
    AlignLeft    = 0x0001,
    AlignRight   = 0x0002,
    AlignHCenter = 0x0004,
    AlignJustify = 0x0008,
}

unsafe impl ExternType for QtAlignmentFlag {
    type Id = type_id!("QtAlignmentFlag");
    type Kind = cxx::kind::Trivial;
}

impl QtAlignmentFlag {
    pub fn contains(&self, flag: QtAlignmentFlag) -> bool {
        (*self as i32 & flag as i32) != 0
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/edit.hpp");

        type QWidget = crate::ui::QWidget;
        type QtAlignmentFlag = super::QtAlignmentFlag;

        unsafe fn new_line_edit(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn line_edit_connect_changed(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
        fn line_edit_get_text(w: &QWidget) -> String;
        fn line_edit_set_text(w: Pin<&mut QWidget>, s: &str);

        fn line_edit_get_alignment(w: &QWidget) -> QtAlignmentFlag;
        fn line_edit_set_alignment(w: Pin<&mut QWidget>, flag: QtAlignmentFlag);

        fn line_edit_is_password(w: &QWidget) -> bool;
        fn line_edit_set_password(w: Pin<&mut QWidget>, v: bool);
    }
}
