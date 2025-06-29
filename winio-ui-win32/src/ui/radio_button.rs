use windows_sys::Win32::UI::{
    Controls::{BST_CHECKED, BST_UNCHECKED, WC_BUTTONW},
    WindowsAndMessaging::{
        BM_GETCHECK, BM_SETCHECK, BN_CLICKED, BS_RADIOBUTTON, SendMessageW, WM_COMMAND, WS_CHILD,
        WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{Point, Size};

use crate::{
    runtime::WindowMessageCommand,
    ui::{Widget, font::measure_string},
};

#[derive(Debug)]
pub struct RadioButton {
    handle: Widget,
}

impl RadioButton {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_RADIOBUTTON as u32,
            0,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        let s = measure_string(self.handle.as_raw_window(), &self.handle.text_u16());
        Size::new(s.width + 18.0, s.height + 2.0)
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn text(&self) -> String {
        self.handle.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(s)
    }

    pub fn is_checked(&self) -> bool {
        unsafe { SendMessageW(self.handle.as_raw_window(), BM_GETCHECK, 0, 0) == BST_CHECKED as _ }
    }

    pub fn set_checked(&self, v: bool) {
        unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                BM_SETCHECK,
                if v { BST_CHECKED } else { BST_UNCHECKED } as _,
                0,
            )
        };
    }

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window()) && (message == BN_CLICKED) {
                self.set_checked(true);
                break;
            }
        }
    }
}
