use cxx::{ExternType, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Edit {
    on_changed: Box<Callback>,
    widget: Widget<ffi::QLineEdit>,
}

#[inherit_methods(from = "self.widget")]
impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_line_edit(parent.as_window().as_qt()) };
        let on_changed = Box::new(Callback::new());
        unsafe {
            ffi::line_edit_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_changed, widget }
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
        self.widget.pin_mut().setText(&s.as_ref().into())
    }

    pub fn is_password(&self) -> bool {
        self.widget.as_ref().echoMode() != QLineEditEchoMode::Normal
    }

    pub fn set_password(&mut self, v: bool) {
        self.widget.pin_mut().setEchoMode(if v {
            QLineEditEchoMode::Password
        } else {
            QLineEditEchoMode::Normal
        });
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

    fn on_changed(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}

winio_handle::impl_as_widget!(Edit, widget);

#[derive(Debug)]
pub struct TextBox {
    on_changed: Box<Callback>,
    widget: Widget<ffi::QTextEdit>,
}

#[inherit_methods(from = "self.widget")]
impl TextBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_text_edit(parent.as_window().as_qt()) };
        let on_changed = Box::new(Callback::new());
        unsafe {
            ffi::text_edit_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_changed, widget }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn text(&self) -> String {
        self.widget.as_ref().toPlainText().into()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.pin_mut().setText(&s.as_ref().into())
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

    fn on_changed(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}

winio_handle::impl_as_widget!(TextBox, widget);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
#[allow(dead_code, clippy::enum_variant_names)]
pub(crate) enum QtAlignmentFlag {
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

#[derive(PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
#[allow(dead_code)]
pub(crate) enum QLineEditEchoMode {
    Normal,
    NoEcho,
    Password,
    PasswordEchoOnEdit,
}

unsafe impl ExternType for QLineEditEchoMode {
    type Id = type_id!("QLineEditEchoMode");
    type Kind = cxx::kind::Trivial;
}

impl_static_cast!(ffi::QLineEdit, ffi::QWidget);

impl_static_cast!(ffi::QTextEdit, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/edit.hpp");

        type QWidget = crate::ui::QWidget;
        type QLineEdit;
        type QTextEdit;
        type QtAlignmentFlag = super::QtAlignmentFlag;
        type QLineEditEchoMode = super::QLineEditEchoMode;
        type QString = crate::ui::QString;

        unsafe fn new_line_edit(parent: *mut QWidget) -> UniquePtr<QLineEdit>;
        unsafe fn line_edit_connect_changed(
            w: Pin<&mut QLineEdit>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn alignment(self: &QLineEdit) -> QtAlignmentFlag;
        fn setAlignment(self: Pin<&mut QLineEdit>, flag: QtAlignmentFlag);
        fn text(self: &QLineEdit) -> QString;
        fn setText(self: Pin<&mut QLineEdit>, s: &QString);
        fn echoMode(self: &QLineEdit) -> QLineEditEchoMode;
        fn setEchoMode(self: Pin<&mut QLineEdit>, m: QLineEditEchoMode);

        unsafe fn new_text_edit(parent: *mut QWidget) -> UniquePtr<QTextEdit>;
        unsafe fn text_edit_connect_changed(
            w: Pin<&mut QTextEdit>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn alignment(self: &QTextEdit) -> QtAlignmentFlag;
        fn setAlignment(self: Pin<&mut QTextEdit>, flag: QtAlignmentFlag);
        fn toPlainText(self: &QTextEdit) -> QString;
        fn setText(self: Pin<&mut QTextEdit>, s: &QString);
    }
}
