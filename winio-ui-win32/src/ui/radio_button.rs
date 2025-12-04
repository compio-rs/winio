use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{BST_CHECKED, BST_UNCHECKED, WC_BUTTONW},
    WindowsAndMessaging::{
        BM_GETCHECK, BM_SETCHECK, BN_CLICKED, BS_RADIOBUTTON, WM_COMMAND, WS_CHILD, WS_TABSTOP,
        WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{Result, runtime::WindowMessageCommand, ui::Widget};

#[derive(Debug)]
pub struct RadioButton {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl RadioButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_RADIOBUTTON as u32,
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
        Ok(Size::new(s.width + 18.0, s.height + 2.0))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool> {
        Ok(self.handle.send_message(BM_GETCHECK, 0, 0) == BST_CHECKED as _)
    }

    fn set_checked_impl(&self, v: bool) {
        self.handle.send_message(
            BM_SETCHECK,
            if v { BST_CHECKED } else { BST_UNCHECKED } as _,
            0,
        );
    }

    pub fn set_checked(&self, v: bool) -> Result<()> {
        self.set_checked_impl(v);
        Ok(())
    }

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_widget().as_win32()) && (message == BN_CLICKED) {
                self.set_checked_impl(true);
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(RadioButton, handle);
