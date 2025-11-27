use std::pin::Pin;

use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Error, GlobalRuntime, Result,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct ListBox {
    on_select: Box<Callback>,
    widget: Widget<ffi::QListWidget>,
}

#[inherit_methods(from = "self.widget")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_list_widget(parent.as_container().as_qt()) }?;
        let on_select = Box::new(Callback::new());
        unsafe {
            ffi::list_widget_connect_select(
                widget.pin_mut(),
                Self::on_select,
                on_select.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { on_select, widget })
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

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        if let Some(item) = unsafe { self.widget.as_ref().item(i as _)?.as_ref() } {
            Ok(item.isSelected()?)
        } else {
            Err(Error::Index(i))
        }
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        unsafe {
            if let Some(item) = self.widget.as_ref().item(i as _)?.as_mut() {
                Pin::new_unchecked(item).setSelected(v)?;
                Ok(())
            } else {
                Err(Error::Index(i))
            }
        }
    }

    fn on_select(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.widget
            .pin_mut()
            .insertItem(i as _, &s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        unsafe {
            let item = self.widget.as_ref().item(i as _)?;
            if item.is_null() {
                return Err(Error::Index(i));
            }
            self.widget.pin_mut().removeItemWidget(item)?;
        }
        Ok(())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        if let Some(item) = unsafe { self.widget.as_ref().item(i as _)?.as_ref() } {
            Ok(item.text()?.try_into()?)
        } else {
            Err(Error::Index(i))
        }
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        unsafe {
            if let Some(item) = self.widget.as_ref().item(i as _)?.as_mut() {
                Pin::new_unchecked(item).setText(&s.as_ref().try_into()?)?;
                Ok(())
            } else {
                Err(Error::Index(i))
            }
        }
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.widget.as_ref().count()? as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.widget.pin_mut().clear()?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(ListBox, widget);

impl_static_cast!(ffi::QListWidget, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/list_box.hpp");

        type QWidget = crate::ui::QWidget;
        type QListWidget;
        type QListWidgetItem;
        type QString = crate::ui::QString;

        unsafe fn new_list_widget(parent: *mut QWidget) -> Result<UniquePtr<QListWidget>>;
        unsafe fn list_widget_connect_select(
            w: Pin<&mut QListWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn item(self: &QListWidget, i: i32) -> Result<*mut QListWidgetItem>;

        fn isSelected(self: &QListWidgetItem) -> Result<bool>;
        fn setSelected(self: Pin<&mut QListWidgetItem>, b: bool) -> Result<()>;
        fn text(self: &QListWidgetItem) -> Result<QString>;
        fn setText(self: Pin<&mut QListWidgetItem>, s: &QString) -> Result<()>;

        fn insertItem(self: Pin<&mut QListWidget>, i: i32, s: &QString) -> Result<()>;
        unsafe fn removeItemWidget(
            self: Pin<&mut QListWidget>,
            item: *mut QListWidgetItem,
        ) -> Result<()>;
        fn clear(self: Pin<&mut QListWidget>) -> Result<()>;
        fn count(self: &QListWidget) -> Result<i32>;
    }
}
