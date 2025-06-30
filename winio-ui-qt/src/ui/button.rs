use std::fmt::Debug;

use cxx::{ExternType, UniquePtr, memory::UniquePtrTarget, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{
        StaticCastTo, Widget, impl_static_cast, impl_static_cast_propogate, static_cast,
        static_cast_mut,
    },
};

pub struct ButtonImpl<T>
where
    T: UniquePtrTarget,
{
    on_click: Box<Callback>,
    widget: Widget<T>,
}

#[allow(private_bounds)]
#[inherit_methods(from = "self.widget")]
impl<T> ButtonImpl<T>
where
    T: StaticCastTo<ffi::QAbstractButton> + StaticCastTo<ffi::QWidget> + UniquePtrTarget,
{
    fn new_impl(mut widget: UniquePtr<T>) -> Self {
        let on_click = Box::new(Callback::new());
        unsafe {
            ffi::push_button_connect_clicked(
                widget.pin_mut().static_cast_mut(),
                Self::on_click,
                on_click.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_click, widget }
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
        static_cast::<ffi::QAbstractButton>(self.widget.as_ref())
            .text()
            .into()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        static_cast_mut::<ffi::QAbstractButton>(self.widget.pin_mut()).setText(&s.as_ref().into());
    }

    fn on_click(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

impl<T: UniquePtrTarget> Debug for ButtonImpl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonImpl")
            .field("on_click", &self.on_click)
            .field("widget", &self.widget)
            .finish()
    }
}

pub type Button = ButtonImpl<ffi::QPushButton>;
pub type CheckBox = ButtonImpl<ffi::QCheckBox>;
pub type RadioButton = ButtonImpl<ffi::QRadioButton>;

impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_push_button(parent.as_window().as_qt()) };
        Self::new_impl(widget)
    }
}

impl CheckBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_check_box(parent.as_window().as_qt()) };
        Self::new_impl(widget)
    }

    pub fn is_checked(&self) -> bool {
        self.widget.as_ref().checkState() != QtCheckState::Unchecked
    }

    pub fn set_checked(&mut self, v: bool) {
        self.widget.pin_mut().setCheckState(if v {
            QtCheckState::Checked
        } else {
            QtCheckState::Unchecked
        });
    }
}

impl RadioButton {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_radio_button(parent.as_window().as_qt()) };
        Self::new_impl(widget)
    }

    pub fn is_checked(&self) -> bool {
        self.widget.as_ref().isChecked()
    }

    pub fn set_checked(&mut self, v: bool) {
        self.widget.pin_mut().setChecked(v);
    }
}

#[doc(hidden)]
#[repr(i32)]
#[allow(dead_code)]
#[derive(PartialEq, Eq)]
pub enum QtCheckState {
    Unchecked,
    PartiallyChecked,
    Checked,
}

unsafe impl ExternType for QtCheckState {
    type Id = type_id!("QtCheckState");
    type Kind = cxx::kind::Trivial;
}

impl_static_cast!(ffi::QAbstractButton, ffi::QWidget);

impl_static_cast!(ffi::QPushButton, ffi::QAbstractButton);

impl_static_cast_propogate!(ffi::QPushButton, ffi::QAbstractButton, ffi::QWidget);

impl_static_cast!(ffi::QCheckBox, ffi::QAbstractButton);

impl_static_cast_propogate!(ffi::QCheckBox, ffi::QAbstractButton, ffi::QWidget);

impl_static_cast!(ffi::QRadioButton, ffi::QAbstractButton);

impl_static_cast_propogate!(ffi::QRadioButton, ffi::QAbstractButton, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/button.hpp");

        type QWidget = crate::ui::QWidget;
        type QString = crate::ui::QString;
        type QAbstractButton;
        type QPushButton;
        type QCheckBox;
        type QRadioButton;
        type QtCheckState = super::QtCheckState;

        unsafe fn new_push_button(parent: *mut QWidget) -> UniquePtr<QPushButton>;
        unsafe fn new_check_box(parent: *mut QWidget) -> UniquePtr<QCheckBox>;
        unsafe fn new_radio_button(parent: *mut QWidget) -> UniquePtr<QRadioButton>;

        unsafe fn push_button_connect_clicked(
            w: Pin<&mut QAbstractButton>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn text(self: &QAbstractButton) -> QString;
        fn setText(self: Pin<&mut QAbstractButton>, s: &QString);

        fn checkState(self: &QCheckBox) -> QtCheckState;
        fn setCheckState(self: Pin<&mut QCheckBox>, s: QtCheckState);

        fn isChecked(self: &QRadioButton) -> bool;
        fn setChecked(self: Pin<&mut QRadioButton>, b: bool);
    }
}
