use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result,
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_combo_box(parent.as_container().as_qt()) }?;
        let on_changed = Box::new(Callback::new());
        let on_select = Box::new(Callback::new());
        unsafe {
            ffi::combo_box_connect_changed(
                widget.pin_mut(),
                Self::on_changed,
                on_changed.as_ref() as *const _ as _,
            )?;
            ffi::combo_box_connect_select(
                widget.pin_mut(),
                Self::on_select,
                on_select.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self {
            on_changed,
            on_select,
            widget,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(self.widget.as_ref().currentText()?.try_into()?)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget
            .pin_mut()
            .setCurrentText(&s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        let i = self.widget.as_ref().currentIndex()?;
        Ok(if i < 0 { None } else { Some(i as _) })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.widget.pin_mut().setCurrentIndex(i as _)?;
        Ok(())
    }

    pub fn is_editable(&self) -> Result<bool> {
        Ok(self.widget.as_ref().isEditable()?)
    }

    pub fn set_editable(&mut self, v: bool) -> Result<()> {
        self.widget.pin_mut().setEditable(v)?;
        Ok(())
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

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        ffi::combo_box_insert(self.widget.pin_mut(), i as _, &s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.widget.pin_mut().removeItem(i as _)?;
        Ok(())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        Ok(self.widget.as_ref().itemText(i as _)?.try_into()?)
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.widget
            .pin_mut()
            .setItemText(i as _, &s.as_ref().try_into()?)?;
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

winio_handle::impl_as_widget!(ComboBox, widget);

impl_static_cast!(ffi::QComboBox, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/combo_box.hpp");

        type QWidget = crate::ui::QWidget;
        type QComboBox;
        type QString = crate::ui::QString;

        unsafe fn new_combo_box(parent: *mut QWidget) -> Result<UniquePtr<QComboBox>>;
        unsafe fn combo_box_connect_changed(
            w: Pin<&mut QComboBox>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn combo_box_connect_select(
            w: Pin<&mut QComboBox>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn currentText(self: &QComboBox) -> Result<QString>;
        fn setCurrentText(self: Pin<&mut QComboBox>, s: &QString) -> Result<()>;

        fn currentIndex(self: &QComboBox) -> Result<i32>;
        fn setCurrentIndex(self: Pin<&mut QComboBox>, i: i32) -> Result<()>;

        fn isEditable(self: &QComboBox) -> Result<bool>;
        fn setEditable(self: Pin<&mut QComboBox>, v: bool) -> Result<()>;

        fn combo_box_insert(w: Pin<&mut QComboBox>, i: i32, s: &QString) -> Result<()>;
        fn removeItem(self: Pin<&mut QComboBox>, i: i32) -> Result<()>;
        fn clear(self: Pin<&mut QComboBox>) -> Result<()>;
        fn count(self: &QComboBox) -> Result<i32>;
        fn itemText(self: &QComboBox, i: i32) -> Result<QString>;
        fn setItemText(self: Pin<&mut QComboBox>, i: i32, s: &QString) -> Result<()>;
    }
}
