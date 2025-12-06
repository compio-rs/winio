use std::pin::Pin;

use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, View,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct TabView {
    on_select: Box<Callback>,
    widget: Widget<ffi::QTabWidget>,
}

#[inherit_methods(from = "self.widget")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_select = Box::new(Callback::new());
        let mut widget = unsafe { ffi::new_tab_widget(parent.as_container().as_qt()) }?;
        unsafe {
            ffi::tab_widget_connect_changed(
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

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        let idx = self.widget.as_ref().currentIndex()?;
        Ok(if idx < 0 { None } else { Some(idx as usize) })
    }

    pub fn set_selection(&mut self, index: usize) -> Result<()> {
        self.widget.pin_mut().setCurrentIndex(index as _)?;
        Ok(())
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

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        unsafe {
            self.widget.pin_mut().insertTab(
                i as _,
                item.view.as_widget().as_qt(),
                &item.text.as_str().try_into()?,
            )?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.widget.pin_mut().removeTab(i as _)?;
        Ok(())
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

winio_handle::impl_as_widget!(TabView, widget);

impl_static_cast!(ffi::QTabWidget, ffi::QWidget);

#[derive(Debug)]
pub struct TabViewItem {
    view: View,
    text: String,
}

#[inherit_methods(from = "self.view")]
impl TabViewItem {
    pub fn new() -> Result<Self> {
        let view = View::new_standalone()?;
        Ok(Self {
            view,
            text: String::new(),
        })
    }

    fn parent(&self) -> Result<*mut ffi::QWidget> {
        Ok(self.view.widget.as_ref().parentWidget()?)
    }

    pub fn text(&self) -> Result<String> {
        unsafe {
            let parent = self.parent()?;
            if !parent.is_null() {
                let parent = Pin::new_unchecked(&mut *(parent.cast::<ffi::QTabWidget>()));
                Ok(parent
                    .tabText(ffi::tab_widget_index_of(
                        &parent,
                        self.view.as_widget().as_qt(),
                    )?)?
                    .try_into()?)
            } else {
                Ok(self.text.clone())
            }
        }
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        unsafe {
            let text = text.as_ref();
            let parent = self.parent()?;
            if !parent.is_null() {
                let parent = Pin::new_unchecked(&mut *(parent.cast::<ffi::QTabWidget>()));
                let index = ffi::tab_widget_index_of(&parent, self.view.as_widget().as_qt())?;
                parent.setTabText(index, &text.try_into()?)?;
            } else {
                self.text = text.into();
            }
        }
        Ok(())
    }

    pub fn size(&self) -> Result<Size>;
}

winio_handle::impl_as_container!(TabViewItem, view);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/tab_view.hpp");

        type QWidget = crate::ui::QWidget;
        type QString = crate::ui::QString;
        type QTabWidget;

        unsafe fn new_tab_widget(parent: *mut QWidget) -> Result<UniquePtr<QTabWidget>>;

        unsafe fn tab_widget_connect_changed(
            w: Pin<&mut QTabWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        unsafe fn tab_widget_index_of(w: &QTabWidget, widget: *mut QWidget) -> Result<i32>;

        fn currentIndex(self: &QTabWidget) -> Result<i32>;
        fn setCurrentIndex(self: Pin<&mut QTabWidget>, index: i32) -> Result<()>;
        fn tabText(self: &QTabWidget, index: i32) -> Result<QString>;
        fn setTabText(self: Pin<&mut QTabWidget>, index: i32, text: &QString) -> Result<()>;

        unsafe fn insertTab(
            self: Pin<&mut QTabWidget>,
            index: i32,
            page: *mut QWidget,
            label: &QString,
        ) -> Result<i32>;
        fn clear(self: Pin<&mut QTabWidget>) -> Result<()>;
        fn count(self: &QTabWidget) -> Result<i32>;
        fn removeTab(self: Pin<&mut QTabWidget>, index: i32) -> Result<()>;
    }
}
