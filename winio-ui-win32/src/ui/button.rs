use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BN_CLICKED, BS_PUSHBUTTON, WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{Result, runtime::WindowMessageCommand, ui::Widget};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON as u32,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let s = self.handle.measure_text()?;
        Ok(Size::new(s.width + 4.0, s.height + 4.0))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_widget().as_win32()) && (message == BN_CLICKED) {
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(Button, handle);
