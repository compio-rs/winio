use crate::{
    AsRawWindow, AsWindow, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    on_changed: Box<Callback>,
    on_select: Box<Callback>,
    widget: Widget,
}

impl<const E: bool> ComboBoxImpl<E> {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_combo_box(parent.as_window().as_raw_window(), E) };
        widget.pin_mut().setVisible(true);
        let on_changed = Box::new(Callback::new());
        let on_select = Box::new(Callback::new());
        unsafe {
            ffi::combo_box_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            );
            ffi::combo_box_connect_select(
                widget.pin_mut(),
                Self::on_select,
                on_select.as_ref() as *const _ as _,
            );
        }
        Self {
            on_changed,
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

    pub fn text(&self) -> String {
        ffi::combo_box_get_text(self.widget.as_ref())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        ffi::combo_box_set_text(self.widget.pin_mut(), s.as_ref())
    }

    pub fn selection(&self) -> Option<usize> {
        let i = ffi::combo_box_get_current_index(self.widget.as_ref());
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = if let Some(i) = i { i as i32 } else { -1 };
        ffi::combo_box_set_current_index(self.widget.pin_mut(), i);
    }

    fn on_select(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(());
        }
    }

    fn on_changed(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(());
        }
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        ffi::combo_box_insert(self.widget.pin_mut(), i as _, s.as_ref());
    }

    pub fn remove(&mut self, i: usize) {
        ffi::combo_box_remove(self.widget.pin_mut(), i as _);
    }

    pub fn get(&self, i: usize) -> String {
        ffi::combo_box_get(self.widget.as_ref(), i as _)
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        ffi::combo_box_set(self.widget.pin_mut(), i as _, s.as_ref());
    }

    pub fn len(&self) -> usize {
        ffi::combo_box_count(self.widget.as_ref()) as _
    }

    pub fn clear(&mut self) {
        ffi::combo_box_clear(self.widget.pin_mut());
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/combo_box.hpp");

        type QWidget = crate::ui::QWidget;

        unsafe fn new_combo_box(parent: *mut QWidget, editable: bool) -> UniquePtr<QWidget>;
        unsafe fn combo_box_connect_changed(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
        unsafe fn combo_box_connect_select(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn combo_box_get_text(w: &QWidget) -> String;
        fn combo_box_set_text(w: Pin<&mut QWidget>, s: &str);

        fn combo_box_get_current_index(w: &QWidget) -> i32;
        fn combo_box_set_current_index(w: Pin<&mut QWidget>, i: i32);

        fn combo_box_insert(w: Pin<&mut QWidget>, i: i32, s: &str);
        fn combo_box_remove(w: Pin<&mut QWidget>, i: i32);
        fn combo_box_clear(w: Pin<&mut QWidget>);
        fn combo_box_count(w: &QWidget) -> i32;
        fn combo_box_get(w: &QWidget, i: i32) -> String;
        fn combo_box_set(w: Pin<&mut QWidget>, i: i32, s: &str);
    }
}
