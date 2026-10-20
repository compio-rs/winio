use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Result, View,
    widgets::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct GroupBox {
    widget: Widget<ffi::QGroupBox>,
    view: View,
}

#[inherit_methods(from = "self.widget")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_group_box(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        let mut view = View::new(&widget)?;
        unsafe {
            ffi::group_box_set_view(widget.pin_mut(), view.widget.pin_mut().get_unchecked_mut())?;
        }
        Ok(Self { widget, view })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let size = ffi::group_box_title_size(self.widget.as_ref())?;
        Ok(Size::new(size.width as _, size.height as _))
    }

    pub fn client_size(&self) -> Result<Size> {
        let size = ffi::group_box_contents_size(self.widget.as_ref())?;
        Ok(Size::new(size.width as _, size.height as _))
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.widget.as_ref().title()?.try_into()?)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.pin_mut().setTitle(&s.as_ref().try_into()?)?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(GroupBox, widget);
winio_handle::impl_as_container!(GroupBox, view);

impl_static_cast!(ffi::QGroupBox, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/group_box.hpp");

        type QWidget = crate::widgets::QWidget;
        type QSize = crate::widgets::QSize;
        type QString = crate::common::QString;
        type QGroupBox;

        unsafe fn new_group_box(parent: *mut QWidget) -> Result<UniquePtr<QGroupBox>>;

        unsafe fn group_box_set_view(w: Pin<&mut QGroupBox>, view: *mut QWidget) -> Result<()>;

        fn group_box_title_size(w: &QGroupBox) -> Result<QSize>;
        fn group_box_contents_size(w: &QGroupBox) -> Result<QSize>;

        fn title(self: &QGroupBox) -> Result<QString>;
        fn setTitle(self: Pin<&mut QGroupBox>, title: &QString) -> Result<()>;
    }
}
