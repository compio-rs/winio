use cxx::{ExternType, type_id};
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsRawContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{
    View,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct ScrollView {
    widget: Widget<ffi::QScrollArea>,
    view: View,
}

#[inherit_methods(from = "self.widget")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Self {
        let widget = unsafe { ffi::new_scroll_area(parent.as_container().as_qt()) };
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        unsafe {
            let mut view = View::new(BorrowedContainer::borrow_raw(widget.as_raw_container()));
            widget
                .pin_mut()
                .setWidget(view.widget.pin_mut().get_unchecked_mut());
            Self { widget, view }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size) {
        self.widget.set_size(s);
        let rect = self.view.widget.as_ref().childrenRect();
        self.view.set_size(Size::new(
            (rect.x2 - rect.x1) as f64,
            (rect.y2 - rect.y1) as f64,
        ));
    }

    pub fn hscroll(&self) -> bool {
        self.widget.as_ref().horizontalScrollBarPolicy() != QtScrollBarPolicy::ScrollBarAlwaysOff
    }

    pub fn set_hscroll(&mut self, v: bool) {
        let policy = if v {
            QtScrollBarPolicy::ScrollBarAsNeeded
        } else {
            QtScrollBarPolicy::ScrollBarAlwaysOff
        };
        self.widget.pin_mut().setHorizontalScrollBarPolicy(policy);
    }

    pub fn vscroll(&self) -> bool {
        self.widget.as_ref().verticalScrollBarPolicy() != QtScrollBarPolicy::ScrollBarAlwaysOff
    }

    pub fn set_vscroll(&mut self, v: bool) {
        let policy = if v {
            QtScrollBarPolicy::ScrollBarAsNeeded
        } else {
            QtScrollBarPolicy::ScrollBarAlwaysOff
        };
        self.widget.pin_mut().setVerticalScrollBarPolicy(policy);
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(ScrollView, widget);
winio_handle::impl_as_container!(ScrollView, view);

impl_static_cast!(ffi::QScrollArea, ffi::QWidget);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
#[allow(dead_code, clippy::enum_variant_names)]
pub(crate) enum QtScrollBarPolicy {
    ScrollBarAsNeeded,
    ScrollBarAlwaysOff,
    ScrollBarAlwaysOn,
}

unsafe impl ExternType for QtScrollBarPolicy {
    type Id = type_id!("QtScrollBarPolicy");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/scroll_view.hpp");

        type QWidget = crate::ui::QWidget;
        type QScrollArea;
        type QtScrollBarPolicy = super::QtScrollBarPolicy;

        unsafe fn new_scroll_area(parent: *mut QWidget) -> UniquePtr<QScrollArea>;

        unsafe fn setWidget(self: Pin<&mut QScrollArea>, widget: *mut QWidget);

        fn horizontalScrollBarPolicy(self: &QScrollArea) -> QtScrollBarPolicy;
        fn setHorizontalScrollBarPolicy(self: Pin<&mut QScrollArea>, policy: QtScrollBarPolicy);
        fn verticalScrollBarPolicy(self: &QScrollArea) -> QtScrollBarPolicy;
        fn setVerticalScrollBarPolicy(self: Pin<&mut QScrollArea>, policy: QtScrollBarPolicy);
    }
}
