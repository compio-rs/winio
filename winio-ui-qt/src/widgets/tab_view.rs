use std::{fmt, pin::Pin};

use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, View,
    widgets::{QWidgetPointer, Widget, impl_static_cast},
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

    pub(crate) fn tab_widget(&self) -> *mut ffi::QTabWidget {
        self.widget.as_ref() as *const _ as *mut _
    }
}

winio_handle::impl_as_widget!(TabView, widget);

impl_static_cast!(ffi::QTabWidget, ffi::QWidget);

pub struct TabViewItem {
    parent: *mut ffi::QTabWidget,
    parent_weak: QWidgetPointer,
    view: View,
}

impl fmt::Debug for TabViewItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabViewItem").finish_non_exhaustive()
    }
}

#[inherit_methods(from = "self.view")]
impl TabViewItem {
    pub fn new(parent: &TabView) -> Result<Self> {
        let view = View::new_standalone()?;
        let parent_weak = parent.widget.weak()?;
        let parent = parent.tab_widget();
        let label: crate::common::QString = "".try_into()?;
        unsafe {
            let mut w = Pin::new_unchecked(&mut *parent);
            w.as_mut().addTab(view.as_widget().as_qt(), &label)?;
        }
        Ok(Self {
            parent,
            parent_weak,
            view,
        })
    }

    pub fn text(&self) -> Result<String> {
        if self.parent_weak.isNull() {
            return Ok(String::new());
        }
        unsafe {
            let parent = Pin::new_unchecked(&mut *self.parent);
            let index = ffi::tab_widget_index_of(&parent, self.view.as_widget().as_qt())?;
            Ok(parent.tabText(index)?.try_into()?)
        }
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let text = text.as_ref();
        if self.parent_weak.isNull() {
            return Ok(());
        }
        unsafe {
            let parent = Pin::new_unchecked(&mut *self.parent);
            let index = ffi::tab_widget_index_of(&parent, self.view.as_widget().as_qt())?;
            parent.setTabText(index, &text.try_into()?)?;
        }
        Ok(())
    }

    pub fn size(&self) -> Result<Size>;
}

impl Drop for TabViewItem {
    fn drop(&mut self) {
        if self.parent_weak.isNull() {
            return;
        }
        unsafe {
            let mut parent = Pin::new_unchecked(&mut *self.parent);
            if let Ok(index) = ffi::tab_widget_index_of(&parent, self.view.as_widget().as_qt())
                && index >= 0
            {
                parent.as_mut().removeTab(index);
            }
        }
    }
}

winio_handle::impl_as_container!(TabViewItem, view);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/tab_view.hpp");

        type QWidget = crate::widgets::QWidget;
        type QString = crate::common::QString;
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

        unsafe fn addTab(
            self: Pin<&mut QTabWidget>,
            page: *mut QWidget,
            label: &QString,
        ) -> Result<i32>;
        fn removeTab(self: Pin<&mut QTabWidget>, index: i32);
    }
}
