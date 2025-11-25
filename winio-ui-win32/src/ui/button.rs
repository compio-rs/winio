use std::io;

use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BN_CLICKED, BS_PUSHBUTTON, WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsRawWindow};
use winio_primitive::{Point, Size};

use crate::{runtime::WindowMessageCommand, ui::Widget};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> io::Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON as u32,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> io::Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> io::Result<()>;

    pub fn is_enabled(&self) -> io::Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> io::Result<()>;

    pub fn preferred_size(&self) -> io::Result<Size> {
        let s = self.handle.measure_text()?;
        Ok(Size::new(s.width + 4.0, s.height + 4.0))
    }

    pub fn loc(&self) -> io::Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> io::Result<()>;

    pub fn size(&self) -> io::Result<Size>;

    pub fn set_size(&mut self, v: Size) -> io::Result<()>;

    pub fn tooltip(&self) -> io::Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> io::Result<()>;

    pub fn text(&self) -> io::Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> io::Result<()>;

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window().as_win32())
                && (message == BN_CLICKED)
            {
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(Button, handle);
