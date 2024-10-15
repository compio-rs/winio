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
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_push_button(parent.as_window().as_raw_window()) };
        widget.pin_mut().show();
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

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/button.hpp");

        type QWidget = crate::ui::QWidget;

        unsafe fn new_push_button(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn push_button_connect_clicked(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
        fn push_button_get_text(w: &QWidget) -> String;
        fn push_button_set_text(w: Pin<&mut QWidget>, s: &str);
    }
}
