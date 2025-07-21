use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct ComboBox {
    on_changed: Box<Callback>,
    on_select: Box<Callback>,
    widget: Widget<ffi::QComboBox>,
}

#[inherit_methods(from = "self.widget")]
impl ComboBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_combo_box(parent.as_window().as_qt()) };
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
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self {
            on_changed,
            on_select,
            widget,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn text(&self) -> String {
        self.widget.as_ref().currentText().into()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.pin_mut().setCurrentText(&s.as_ref().into());
    }

    pub fn selection(&self) -> Option<usize> {
        let i = self.widget.as_ref().currentIndex();
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = if let Some(i) = i { i as i32 } else { -1 };
        self.widget.pin_mut().setCurrentIndex(i);
    }

    pub fn is_editable(&self) -> bool {
        self.widget.as_ref().isEditable()
    }

    pub fn set_editable(&mut self, v: bool) {
        self.widget.pin_mut().setEditable(v);
    }

    fn on_select(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    fn on_changed(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        ffi::combo_box_insert(self.widget.pin_mut(), i as _, &s.as_ref().into());
    }

    pub fn remove(&mut self, i: usize) {
        self.widget.pin_mut().removeItem(i as _);
    }

    pub fn get(&self, i: usize) -> String {
        self.widget.as_ref().itemText(i as _).into()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        self.widget
            .pin_mut()
            .setItemText(i as _, &s.as_ref().into());
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

winio_handle::impl_as_widget!(ComboBox, widget);

impl_static_cast!(ffi::QComboBox, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/combo_box.hpp");

        type QWidget = crate::ui::QWidget;
        type QComboBox;
        type QString = crate::ui::QString;

        unsafe fn new_combo_box(parent: *mut QWidget) -> UniquePtr<QComboBox>;
        unsafe fn combo_box_connect_changed(
            w: Pin<&mut QComboBox>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
        unsafe fn combo_box_connect_select(
            w: Pin<&mut QComboBox>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn currentText(self: &QComboBox) -> QString;
        fn setCurrentText(self: Pin<&mut QComboBox>, s: &QString);

        fn currentIndex(self: &QComboBox) -> i32;
        fn setCurrentIndex(self: Pin<&mut QComboBox>, i: i32);

        fn isEditable(self: &QComboBox) -> bool;
        fn setEditable(self: Pin<&mut QComboBox>, v: bool);

        fn combo_box_insert(w: Pin<&mut QComboBox>, i: i32, s: &QString);
        fn removeItem(self: Pin<&mut QComboBox>, i: i32);
        fn clear(self: Pin<&mut QComboBox>);
        fn count(self: &QComboBox) -> i32;
        fn itemText(self: &QComboBox, i: i32) -> QString;
        fn setItemText(self: Pin<&mut QComboBox>, i: i32, s: &QString);
    }
}
