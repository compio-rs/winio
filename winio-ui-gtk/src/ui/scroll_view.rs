use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::{Result, Widget};

#[derive(Debug)]
pub struct ScrollView {
    swindow: gtk4::ScrolledWindow,
    fixed: gtk4::Fixed,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let fixed = gtk4::Fixed::new();
        let swindow = gtk4::ScrolledWindow::new();
        let handle = Widget::new(parent, unsafe { swindow.clone().unsafe_cast() })?;
        swindow.set_child(Some(&fixed));
        swindow.set_hscrollbar_policy(gtk4::PolicyType::Automatic);
        swindow.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        Ok(Self {
            swindow,
            fixed,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.handle.set_size(s)?;
        self.swindow.set_max_content_height(s.height as _);
        self.swindow.set_max_content_width(s.width as _);
        Ok(())
    }

    pub fn hscroll(&self) -> Result<bool> {
        Ok(self.swindow.hscrollbar_policy() == gtk4::PolicyType::Automatic)
    }

    pub fn set_hscroll(&mut self, v: bool) -> Result<()> {
        self.swindow.set_hscrollbar_policy(if v {
            gtk4::PolicyType::Automatic
        } else {
            gtk4::PolicyType::External
        });
        Ok(())
    }

    pub fn vscroll(&self) -> Result<bool> {
        Ok(self.swindow.vscrollbar_policy() == gtk4::PolicyType::Automatic)
    }

    pub fn set_vscroll(&mut self, v: bool) -> Result<()> {
        self.swindow.set_vscrollbar_policy(if v {
            gtk4::PolicyType::Automatic
        } else {
            gtk4::PolicyType::External
        });
        Ok(())
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
