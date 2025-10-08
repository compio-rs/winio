use std::pin::Pin;

use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawWidget};
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, View,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct TabView {
    on_select: Box<Callback>,
    widget: Widget<ffi::QTabWidget>,
}

#[inherit_methods(from = "self.widget")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Self {
        let on_select = Box::new(Callback::new());
        let mut widget = unsafe { ffi::new_tab_widget(parent.as_container().as_qt()) };
        unsafe {
            ffi::tab_widget_connect_changed(
                widget.pin_mut(),
                Self::on_select,
                on_select.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_select, widget }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn selection(&self) -> Option<usize> {
        let idx = self.widget.as_ref().currentIndex();
        if idx < 0 { None } else { Some(idx as usize) }
    }

    pub fn set_selection(&mut self, index: usize) {
        self.widget.pin_mut().setCurrentIndex(index as _);
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

    pub fn insert(&mut self, i: usize, item: &TabViewItem) {
        unsafe {
            self.widget.pin_mut().insertTab(
                i as _,
                item.view.as_raw_widget().as_qt(),
                &item.text.as_str().into(),
            );
        }
    }

    pub fn remove(&mut self, i: usize) {
        self.widget.pin_mut().removeTab(i as _);
    }

    pub fn len(&self) -> usize {
        self.widget.as_ref().count() as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.widget.pin_mut().clear();
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
    pub fn new(_parent: &TabView) -> Self {
        let view = View::new_standalone();
        Self {
            view,
            text: String::new(),
        }
    }

    fn parent(&self) -> *mut ffi::QWidget {
        self.view.widget.as_ref().parentWidget()
    }

    pub fn text(&self) -> String {
        unsafe {
            let parent = self.parent();
            if !parent.is_null() {
                let parent = Pin::new_unchecked(&mut *(parent.cast::<ffi::QTabWidget>()));
                parent
                    .tabText(ffi::tab_widget_index_of(
                        &parent,
                        self.view.as_raw_widget().as_qt(),
                    ))
                    .into()
            } else {
                self.text.clone()
            }
        }
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        unsafe {
            let text = text.as_ref();
            let parent = self.parent();
            if !parent.is_null() {
                let parent = Pin::new_unchecked(&mut *(parent.cast::<ffi::QTabWidget>()));
                let index = ffi::tab_widget_index_of(&parent, self.view.as_raw_widget().as_qt());
                parent.setTabText(index, &text.into());
            } else {
                self.text = text.into();
            }
        }
    }

    pub fn size(&self) -> Size;
}

winio_handle::impl_as_container!(TabViewItem, view);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/tab_view.hpp");

        type QWidget = crate::ui::QWidget;
        type QString = crate::ui::QString;
        type QTabWidget;

        unsafe fn new_tab_widget(parent: *mut QWidget) -> UniquePtr<QTabWidget>;

        unsafe fn tab_widget_connect_changed(
            w: Pin<&mut QTabWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        unsafe fn tab_widget_index_of(w: &QTabWidget, widget: *mut QWidget) -> i32;

        fn currentIndex(self: &QTabWidget) -> i32;
        fn setCurrentIndex(self: Pin<&mut QTabWidget>, index: i32);
        fn tabText(self: &QTabWidget, index: i32) -> QString;
        fn setTabText(self: Pin<&mut QTabWidget>, index: i32, text: &QString);

        unsafe fn insertTab(
            self: Pin<&mut QTabWidget>,
            index: i32,
            page: *mut QWidget,
            label: &QString,
        ) -> i32;
        fn clear(self: Pin<&mut QTabWidget>);
        fn count(self: &QTabWidget) -> i32;
        fn removeTab(self: Pin<&mut QTabWidget>, index: i32);
    }
}
