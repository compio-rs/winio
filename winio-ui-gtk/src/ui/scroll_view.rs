use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::Widget;

#[derive(Debug)]
pub struct ScrollView {
    swindow: gtk4::ScrolledWindow,
    fixed: gtk4::Fixed,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Self {
        let fixed = gtk4::Fixed::new();
        let swindow = gtk4::ScrolledWindow::new();
        let handle = Widget::new(parent, unsafe { swindow.clone().unsafe_cast() });
        swindow.set_child(Some(&fixed));
        swindow.set_hscrollbar_policy(gtk4::PolicyType::Automatic);
        swindow.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        Self {
            swindow,
            fixed,
            handle,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size) {
        self.handle.set_size(s);
        self.swindow.set_max_content_height(s.height as _);
        self.swindow.set_max_content_width(s.width as _);
    }

    pub fn hscroll(&self) -> bool {
        self.swindow.hscrollbar_policy() == gtk4::PolicyType::Automatic
    }

    pub fn set_hscroll(&mut self, v: bool) {
        self.swindow.set_hscrollbar_policy(if v {
            gtk4::PolicyType::Automatic
        } else {
            gtk4::PolicyType::External
        });
    }

    pub fn vscroll(&self) -> bool {
        self.swindow.vscrollbar_policy() == gtk4::PolicyType::Automatic
    }

    pub fn set_vscroll(&mut self, v: bool) {
        self.swindow.set_vscrollbar_policy(if v {
            gtk4::PolicyType::Automatic
        } else {
            gtk4::PolicyType::External
        });
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);

impl AsRawContainer for ScrollView {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Gtk(self.fixed.clone())
    }
}

winio_handle::impl_as_container!(ScrollView);
