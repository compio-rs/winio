use std::{
    io,
    mem::ManuallyDrop,
    rc::{Rc, Weak},
};

use super::Widget;
use crate::{Callback, Point, Size};

pub struct Edit {
    widget: Widget,
    on_changed: Callback,
}

impl Edit {
    pub fn new(parent: &Widget) -> io::Result<Rc<Self>> {
        let mut widget = parent.pin_mut(ffi::new_line_edit);
        widget.pin_mut().show();
        let widget = Rc::new_cyclic(|this: &Weak<Self>| {
            unsafe {
                ffi::line_edit_connect_changed(
                    widget.pin_mut(),
                    Self::on_changed,
                    this.clone().into_raw().cast(),
                );
            }
            Self {
                widget: Widget::new(widget),
                on_changed: Callback::new(),
            }
        });
        Ok(widget)
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(self.widget.loc())
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.widget.set_loc(p);
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(self.widget.size())
    }

    pub fn set_size(&self, s: Size) -> io::Result<()> {
        self.widget.set_size(s);
        Ok(())
    }

    pub fn text(&self) -> io::Result<String> {
        Ok(self.widget.as_ref(ffi::line_edit_get_text))
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.widget
            .pin_mut(|w| ffi::line_edit_set_text(w, s.as_ref()));
        Ok(())
    }

    fn on_changed(this: *const u8) {
        let this = ManuallyDrop::new(unsafe { Weak::<Self>::from_raw(this.cast()) });
        if let Some(this) = this.upgrade() {
            this.on_changed.signal(());
        }
    }

    pub async fn wait_changed(&self) {
        self.on_changed.wait().await
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/edit.hpp");

        type QWidget = crate::QWidget;

        fn new_line_edit(parent: Pin<&mut QWidget>) -> UniquePtr<QWidget>;
        unsafe fn line_edit_connect_changed(
            w: Pin<&mut QWidget>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );
        fn line_edit_get_text(w: &QWidget) -> String;
        fn line_edit_set_text(w: Pin<&mut QWidget>, s: &str);
    }
}
