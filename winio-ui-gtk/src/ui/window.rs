use std::rc::Rc;

use gtk4::{glib::Propagation, prelude::*};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWindow, AsWindow, BorrowedContainer, BorrowedWindow,
    RawContainer, RawWindow,
};
use winio_primitive::{ColorTheme, Point, Size};

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct Window {
    on_size: Rc<Callback<()>>,
    on_close: Rc<Callback<()>>,
    window: gtk4::Window,
    swindow: gtk4::ScrolledWindow,
    fixed: gtk4::Fixed,
}

impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let window = gtk4::Window::new();

        let color = window.color();
        let brightness = color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114;
        let theme = if brightness > 0.5 {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        };
        super::COLOR_THEME.set(theme);

        let swindow = gtk4::ScrolledWindow::new();
        swindow.set_hscrollbar_policy(gtk4::PolicyType::External);
        swindow.set_vscrollbar_policy(gtk4::PolicyType::External);
        let fixed = gtk4::Fixed::new();
        window.set_child(Some(&swindow));
        swindow.set_child(Some(&fixed));

        let on_size = Rc::new(Callback::new());
        let on_close = Rc::new(Callback::new());

        window.connect_default_width_notify({
            let on_size = on_size.clone();
            move |_| {
                on_size.signal::<GlobalRuntime>(());
            }
        });
        window.connect_default_height_notify({
            let on_size = on_size.clone();
            move |_| {
                on_size.signal::<GlobalRuntime>(());
            }
        });
        window.connect_state_flags_changed({
            let on_size = on_size.clone();
            move |_, _| {
                on_size.signal::<GlobalRuntime>(());
            }
        });
        window.connect_close_request({
            let on_close = on_close.clone();
            move |_| {
                if !on_close.signal::<GlobalRuntime>(()) {
                    return Propagation::Stop;
                }
                Propagation::Proceed
            }
        });
        Self {
            on_size,
            on_close,
            window,
            swindow,
            fixed,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.window.get_visible()
    }

    pub fn set_visible(&self, v: bool) {
        if v {
            self.window.present();
        } else {
            self.window.set_visible(v);
        }
    }

    pub fn loc(&self) -> Point {
        Point::zero()
    }

    pub fn set_loc(&mut self, _p: Point) {}

    pub fn size(&self) -> Size {
        let mut width = self.window.width();
        if width == 0 {
            width = self.window.default_width();
        }
        let mut height = self.window.height();
        if height == 0 {
            height = self.window.default_height();
        }
        Size::new(width as _, height as _)
    }

    pub fn set_size(&mut self, s: Size) {
        self.window.set_default_size(s.width as _, s.height as _);
    }

    pub fn client_size(&self) -> Size {
        let mut width = self.swindow.width();
        if width == 0 {
            width = self.window.default_width();
        }
        let mut height = self.swindow.height();
        if height == 0 {
            height = self.window.default_height();
        }
        Size::new(width as _, height as _)
    }

    pub fn text(&self) -> String {
        self.window
            .title()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.window.set_title(Some(s.as_ref()));
    }

    pub async fn wait_size(&self) {
        self.on_size.wait().await
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::Gtk(self.window.clone())
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.as_raw_window()) }
    }
}

impl AsRawContainer for Window {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Gtk(self.fixed.clone())
    }
}

impl AsContainer for Window {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}

#[derive(Debug)]
pub struct View {
    fixed: gtk4::Fixed,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Self {
        let fixed = gtk4::Fixed::new();
        let handle = Widget::new(parent, unsafe { fixed.clone().unsafe_cast() });
        Self { fixed, handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);
}

winio_handle::impl_as_widget!(View, handle);

impl AsRawContainer for View {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Gtk(self.fixed.clone())
    }
}

impl AsContainer for View {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}
