use std::pin::Pin;

use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct ListBox {
    on_select: Box<Callback>,
    widget: Widget<ffi::QListWidget>,
}

#[inherit_methods(from = "self.widget")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Self {
        let mut widget = unsafe { ffi::new_list_widget(parent.as_container().as_qt()) };
        let on_select = Box::new(Callback::new());
        unsafe {
            ffi::list_widget_connect_select(
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

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn is_selected(&self, i: usize) -> bool {
        unsafe { self.widget.as_ref().item(i as _).as_ref() }
            .map(|item| item.isSelected())
            .unwrap_or_default()
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        unsafe {
            if let Some(item) = self.widget.as_ref().item(i as _).as_mut() {
                Pin::new_unchecked(item).setSelected(v)
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

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        self.widget.pin_mut().insertItem(i as _, &s.as_ref().into());
    }

    pub fn remove(&mut self, i: usize) {
        unsafe {
            let item = self.widget.as_ref().item(i as _);
            self.widget.pin_mut().removeItemWidget(item);
        }
    }

    pub fn get(&self, i: usize) -> String {
        unsafe { self.widget.as_ref().item(i as _).as_ref() }
            .map(|item| item.text().into())
            .unwrap_or_default()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        unsafe {
            if let Some(item) = self.widget.as_ref().item(i as _).as_mut() {
                Pin::new_unchecked(item).setText(&s.as_ref().into());
            }
        }
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
        unsafe fn new_list_widget(parent: *mut QWidget) -> UniquePtr<QListWidget>;
        unsafe fn list_widget_connect_select(
            w: Pin<&mut QListWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn item(self: &QListWidget, i: i32) -> *mut QListWidgetItem;

        fn isSelected(self: &QListWidgetItem) -> bool;
        fn setSelected(self: Pin<&mut QListWidgetItem>, b: bool);
        fn text(self: &QListWidgetItem) -> QString;
        fn setText(self: Pin<&mut QListWidgetItem>, s: &QString);

        fn insertItem(self: Pin<&mut QListWidget>, i: i32, s: &QString);
        unsafe fn removeItemWidget(self: Pin<&mut QListWidget>, item: *mut QListWidgetItem);
        fn clear(self: Pin<&mut QListWidget>);
        fn count(self: &QListWidget) -> i32;
    }
}
