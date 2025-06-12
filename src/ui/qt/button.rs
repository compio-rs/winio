use cxx::UniquePtr;

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct Button {
    on_click: Box<Callback>,
    widget: Widget,
}

impl Button {
    fn new_impl(mut widget: UniquePtr<ffi::QWidget>) -> Self {
        widget.pin_mut().setVisible(true);
        let on_click = Box::new(Callback::new());
        unsafe {
            ffi::push_button_connect_clicked(
                widget.pin_mut(),
                Self::on_click,
                on_click.as_ref() as *const _ as _,
            );
        }
        Self {
            on_click,
            widget: Widget::new(widget),
        }
    }

    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_push_button(parent.as_window().as_raw_window()) };
        Self::new_impl(widget)
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
        ffi::push_button_get_text(self.widget.as_ref())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        ffi::push_button_set_text(self.widget.pin_mut(), s.as_ref())
    }

    fn on_click(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal(());
        }
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

#[derive(Debug)]
pub struct CheckBox {
    widget: Button,
}

impl CheckBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_check_box(parent.as_window().as_raw_window()) };
        let widget = Button::new_impl(widget);
        Self { widget }
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
        self.widget.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s);
    }

    pub fn is_checked(&self) -> bool {
        ffi::check_box_is_checked(self.widget.widget.as_ref())
    }

    pub fn set_checked(&mut self, v: bool) {
        ffi::check_box_set_checked(self.widget.widget.pin_mut(), v);
    }

    pub async fn wait_click(&self) {
        self.widget.wait_click().await
    }
}

#[derive(Debug)]
pub struct RadioButton {
    widget: Button,
}

impl RadioButton {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = unsafe { ffi::new_radio_button(parent.as_window().as_raw_window()) };
        let widget = Button::new_impl(widget);
        Self { widget }
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
        self.widget.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s);
    }

    pub fn is_checked(&self) -> bool {
        ffi::radio_button_is_checked(self.widget.widget.as_ref())
    }

    pub fn set_checked(&mut self, v: bool) {
        ffi::radio_button_set_checked(self.widget.widget.pin_mut(), v);
    }

    pub async fn wait_click(&self) {
        self.widget.wait_click().await
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/button.hpp");

        type QWidget = crate::ui::QWidget;

        unsafe fn new_push_button(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn new_check_box(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn new_radio_button(parent: *mut QWidget) -> UniquePtr<QWidget>;

        unsafe fn push_button_connect_clicked(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn push_button_get_text(w: &QWidget) -> String;
        fn push_button_set_text(w: Pin<&mut QWidget>, s: &str);

        fn check_box_is_checked(w: &QWidget) -> bool;
        fn check_box_set_checked(w: Pin<&mut QWidget>, v: bool);

        fn radio_button_is_checked(w: &QWidget) -> bool;
        fn radio_button_set_checked(w: Pin<&mut QWidget>, v: bool);
    }
}
