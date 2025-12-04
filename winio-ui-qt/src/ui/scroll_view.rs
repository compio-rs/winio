use cxx::{ExternType, type_id};
use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Result, View,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct ScrollView {
    widget: Widget<ffi::QScrollArea>,
    view: View,
}

#[inherit_methods(from = "self.widget")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_scroll_area(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        unsafe {
            let mut view = View::new(&widget)?;
            widget
                .pin_mut()
                .setWidget(view.widget.pin_mut().get_unchecked_mut())?;
            Ok(Self { widget, view })
        }
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.widget.set_size(s)?;
        let rect = self.view.widget.as_ref().childrenRect()?;
        self.view.set_size(Size::new(
            (rect.x2 - rect.x1) as f64,
            (rect.y2 - rect.y1) as f64,
        ))
    }

    pub fn hscroll(&self) -> Result<bool> {
        Ok(self.widget.as_ref().horizontalScrollBarPolicy()?
            != QtScrollBarPolicy::ScrollBarAlwaysOff)
    }

    pub fn set_hscroll(&mut self, v: bool) -> Result<()> {
        let policy = if v {
            QtScrollBarPolicy::ScrollBarAsNeeded
        } else {
            QtScrollBarPolicy::ScrollBarAlwaysOff
        };
        self.widget.pin_mut().setHorizontalScrollBarPolicy(policy)?;
        Ok(())
    }

    pub fn vscroll(&self) -> Result<bool> {
        Ok(
            self.widget.as_ref().verticalScrollBarPolicy()?
                != QtScrollBarPolicy::ScrollBarAlwaysOff,
        )
    }

    pub fn set_vscroll(&mut self, v: bool) -> Result<()> {
        let policy = if v {
            QtScrollBarPolicy::ScrollBarAsNeeded
        } else {
            QtScrollBarPolicy::ScrollBarAlwaysOff
        };
        self.widget.pin_mut().setVerticalScrollBarPolicy(policy)?;
        Ok(())
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

        unsafe fn new_scroll_area(parent: *mut QWidget) -> Result<UniquePtr<QScrollArea>>;

        unsafe fn setWidget(self: Pin<&mut QScrollArea>, widget: *mut QWidget) -> Result<()>;

        fn horizontalScrollBarPolicy(self: &QScrollArea) -> Result<QtScrollBarPolicy>;
        fn setHorizontalScrollBarPolicy(
            self: Pin<&mut QScrollArea>,
            policy: QtScrollBarPolicy,
        ) -> Result<()>;
        fn verticalScrollBarPolicy(self: &QScrollArea) -> Result<QtScrollBarPolicy>;
        fn setVerticalScrollBarPolicy(
            self: Pin<&mut QScrollArea>,
            policy: QtScrollBarPolicy,
        ) -> Result<()>;
    }
}
