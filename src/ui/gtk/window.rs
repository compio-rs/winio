use std::{io, rc::Rc};

use glib::object::Cast;

pub trait AsWidget {
    fn as_widget(&self) -> &gtk4::Widget;
}

impl<T: AsWidget> AsWidget for &'_ T {
    fn as_widget(&self) -> &gtk4::Widget {
        (**self).as_widget()
    }
}

impl<T: AsWidget> AsWidget for Rc<T> {
    fn as_widget(&self) -> &gtk4::Widget {
        (**self).as_widget()
    }
}

pub struct Window {
    window: gtk4::Window,
}

impl Window {
    pub fn new() -> io::Result<Rc<Self>> {
        let window = gtk4::Window::new();
        Ok(Rc::new(Self { window }))
    }
}

impl AsWidget for Window {
    fn as_widget(&self) -> &gtk4::Widget {
        unsafe { self.window.unsafe_cast_ref() }
    }
}
