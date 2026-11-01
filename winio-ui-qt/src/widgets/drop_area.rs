use std::fmt::Debug;

use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result,
    platform::Data,
    widgets::{Widget, impl_static_cast},
};

/// A widget that receives drag-and-drop data.
#[derive(Debug)]
pub struct DropArea {
    on_drop: Box<Callback<Data>>,
    widget: Widget<ffi::DropAreaWidget>,
}

#[inherit_methods(from = "self.widget")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_drop = Box::new(Callback::new());
        let widget = unsafe {
            ffi::new_drop_area(
                parent.as_container().as_qt(),
                Self::on_drop,
                on_drop.as_ref() as *const _ as _,
            )?
        };
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { on_drop, widget })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    fn on_drop(c: *const u8, data: &ffi::QMimeData) {
        let c = c as *const Callback<Data>;
        if let Some(c) = unsafe { c.as_ref() } {
            match Data::from_mime_data(data) {
                Ok(data) => c.signal::<GlobalRuntime>(data),
                Err(e) => compio_log::error!("Failed to read dropped data: {e:?}"),
            }
        }
    }

    pub async fn wait_drop(&self) -> Data {
        self.on_drop.wait().await
    }
}

winio_handle::impl_as_widget!(DropArea, widget);
winio_handle::impl_as_container!(DropArea, widget);

impl_static_cast!(ffi::DropAreaWidget, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/drop_area.hpp");

        type QWidget = crate::widgets::QWidget;
        type QMimeData = crate::platform::QMimeData;
        type DropAreaWidget;

        unsafe fn new_drop_area(
            parent: *mut QWidget,
            callback: unsafe fn(*const u8, &QMimeData),
            data: *const u8,
        ) -> Result<UniquePtr<DropAreaWidget>>;
    }
}
