use crate::{
    AsRawWindow, AsWindow, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct ListBox {
    on_select: Box<Callback>,
    widget: Widget,
}

impl ListBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_list_widget(parent.as_window().as_raw_window()) };
        widget.pin_mut().setVisible(true);
        let on_select = Box::new(Callback::new());
        unsafe {
            ffi::list_widget_connect_select(
                widget.pin_mut(),
                Self::on_select,
                on_select.as_ref() as *const _ as _,
            );
        }
        Self {
            on_select,
            widget: Widget::new(widget),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.widget.preferred_size()
    }

    pub fn min_size(&self) -> Size {
        self.widget.min_size()
    }

    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.widget.size()
    }

    pub fn set_size(&mut self, s: Size) {
        self.widget.set_size(s);
    }

    pub fn is_selected(&self, i: usize) -> bool {
        ffi::list_widget_is_selected(self.widget.as_ref(), i as _)
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        ffi::list_widget_set_selected(self.widget.pin_mut(), i as _, v);
    }

    fn on_select(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(());
        }
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        ffi::list_widget_insert(self.widget.pin_mut(), i as _, s.as_ref());
    }

    pub fn remove(&mut self, i: usize) {
        ffi::list_widget_remove(self.widget.pin_mut(), i as _);
    }

    pub fn get(&self, i: usize) -> String {
        ffi::list_widget_get(self.widget.as_ref(), i as _)
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        ffi::list_widget_set(self.widget.pin_mut(), i as _, s.as_ref());
    }

    pub fn len(&self) -> usize {
        ffi::list_widget_count(self.widget.as_ref()) as _
    }

    pub fn clear(&mut self) {
        ffi::list_widget_clear(self.widget.pin_mut());
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/list_box.hpp");

        type QWidget = crate::ui::QWidget;

        unsafe fn new_list_widget(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn list_widget_connect_select(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn list_widget_is_selected(w: &QWidget, i: i32) -> bool;
        fn list_widget_set_selected(w: Pin<&mut QWidget>, i: i32, v: bool);

        fn list_widget_insert(w: Pin<&mut QWidget>, i: i32, s: &str);
        fn list_widget_remove(w: Pin<&mut QWidget>, i: i32);
        fn list_widget_clear(w: Pin<&mut QWidget>);
        fn list_widget_count(w: &QWidget) -> i32;
        fn list_widget_get(w: &QWidget, i: i32) -> String;
        fn list_widget_set(w: Pin<&mut QWidget>, i: i32, s: &str);
    }
}
