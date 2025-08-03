use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{BST_CHECKED, BST_UNCHECKED, WC_BUTTONW},
    WindowsAndMessaging::{
        BM_GETCHECK, BM_SETCHECK, BN_CLICKED, BS_CHECKBOX, WM_COMMAND, WS_CHILD, WS_TABSTOP,
        WS_VISIBLE,
    },
};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{Point, Size};

use crate::{runtime::WindowMessageCommand, ui::Widget};

#[derive(Debug)]
pub struct CheckBox {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl CheckBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX as u32,
            0,
            parent.as_window().as_win32(),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let s = self.handle.measure_text();
        Size::new(s.width + 18.0, s.height + 2.0)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn is_checked(&self) -> bool {
        self.handle.send_message(BM_GETCHECK, 0, 0) == BST_CHECKED as _
    }

    pub fn set_checked(&self, v: bool) {
        self.handle.send_message(
            BM_SETCHECK,
            if v { BST_CHECKED } else { BST_UNCHECKED } as _,
            0,
        );
    }

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window().as_win32())
                && (message == BN_CLICKED)
            {
                self.set_checked(!self.is_checked());
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(CheckBox, handle);
