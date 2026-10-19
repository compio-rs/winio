use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BS_GROUPBOX, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT, WS_VISIBLE,
    },
};
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, View, widgets::Widget};

const PADDING: f64 = 3.0;

#[derive(Debug)]
pub struct GroupBox {
    handle: Widget,
    view: View,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | BS_GROUPBOX as u32,
            WS_EX_CONTROLPARENT,
            parent.as_container().as_win32(),
        )?;
        let view = View::new(&handle)?;
        let mut this = Self { handle, view };
        this.update_view()?;
        Ok(this)
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.update_view()
    }

    pub fn preferred_size(&self) -> Result<Size> {
        self.handle.measure_text()
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn update_view(&mut self) -> Result<()> {
        let size = self.handle.size()?;
        let title_height = self.handle.measure_text()?.height;
        let top = title_height + PADDING;
        let width = (size.width - PADDING * 2.0).max(0.0);
        let height = (size.height - top - PADDING).max(0.0);
        self.view.set_loc(Point::new(PADDING, top))?;
        self.view.set_size(Size::new(width, height))
    }
}

winio_handle::impl_as_widget!(GroupBox, handle);
winio_handle::impl_as_container!(GroupBox, view);
