use cxx::{ExternType, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{
    GlobalRuntime, Result,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Edit {
    on_changed: Box<Callback>,
    widget: Widget<ffi::QLineEdit>,
}

#[inherit_methods(from = "self.widget")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_line_edit(parent.as_container().as_qt()) }?;
        let on_changed = Box::new(Callback::new());
        unsafe {
            ffi::line_edit_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { on_changed, widget })
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

    pub fn is_password(&self) -> Result<bool> {
        Ok(self.widget.as_ref().echoMode()? != QLineEditEchoMode::Normal)
    }

    pub fn set_password(&mut self, v: bool) -> Result<()> {
        if v {
            self.set_readonly(false)?;
        }
        self.widget.pin_mut().setEchoMode(if v {
            QLineEditEchoMode::Password
        } else {
            QLineEditEchoMode::Normal
        })?;
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

    pub fn is_readonly(&self) -> Result<bool> {
        if self.is_password()? {
            Ok(false)
        } else {
            Ok(self.widget.as_ref().isReadOnly()?)
        }
    }

    pub fn set_readonly(&mut self, r: bool) -> Result<()> {
        if !self.is_password()? {
            self.widget.pin_mut().setReadOnly(r)?;
        }
        Ok(())
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_text_edit(parent.as_container().as_qt()) }?;
        let on_changed = Box::new(Callback::new());
        unsafe {
            ffi::text_edit_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { on_changed, widget })
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

    pub fn text(&self) -> Result<String> {
        Ok(self.widget.as_ref().toPlainText()?.try_into()?)
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

    pub fn is_readonly(&self) -> Result<bool> {
        Ok(self.widget.as_ref().isReadOnly()?)
    }

    pub fn set_readonly(&mut self, r: bool) -> Result<()> {
        self.widget.pin_mut().setReadOnly(r)?;
        Ok(())
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

bitflags::bitflags! {
    pub struct QtAlignmentFlag: i32 {
        const AlignLeft    = 0x0001;
        const AlignRight   = 0x0002;
        const AlignHCenter = 0x0004;
        const AlignJustify = 0x0008;

        const _ = !0;
    }
}

unsafe impl ExternType for QtAlignmentFlag {
    type Id = type_id!("QtAlignmentFlag");
    type Kind = cxx::kind::Trivial;
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

        unsafe fn new_line_edit(parent: *mut QWidget) -> Result<UniquePtr<QLineEdit>>;
        unsafe fn line_edit_connect_changed(
            w: Pin<&mut QLineEdit>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn alignment(self: &QLineEdit) -> Result<QtAlignmentFlag>;
        fn setAlignment(self: Pin<&mut QLineEdit>, flag: QtAlignmentFlag) -> Result<()>;
        fn text(self: &QLineEdit) -> Result<QString>;
        fn setText(self: Pin<&mut QLineEdit>, s: &QString) -> Result<()>;
        fn echoMode(self: &QLineEdit) -> Result<QLineEditEchoMode>;
        fn setEchoMode(self: Pin<&mut QLineEdit>, m: QLineEditEchoMode) -> Result<()>;
        fn isReadOnly(self: &QLineEdit) -> Result<bool>;
        fn setReadOnly(self: Pin<&mut QLineEdit>, r: bool) -> Result<()>;

        unsafe fn new_text_edit(parent: *mut QWidget) -> Result<UniquePtr<QTextEdit>>;
        unsafe fn text_edit_connect_changed(
            w: Pin<&mut QTextEdit>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn alignment(self: &QTextEdit) -> Result<QtAlignmentFlag>;
        fn setAlignment(self: Pin<&mut QTextEdit>, flag: QtAlignmentFlag) -> Result<()>;
        fn toPlainText(self: &QTextEdit) -> Result<QString>;
        fn setText(self: Pin<&mut QTextEdit>, s: &QString) -> Result<()>;
        fn isReadOnly(self: &QTextEdit) -> Result<bool>;
        fn setReadOnly(self: Pin<&mut QTextEdit>, r: bool) -> Result<()>;
    }
}
