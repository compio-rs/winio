use raw_window_handle::HasWindowHandle;
use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BN_CLICKED, BS_PUSHBUTTON, WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{
    Point, Size,
    ui::{Widget, unwrap_win32_handle},
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
}

impl Button {
    pub fn new(parent: impl HasWindowHandle) -> Self {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON as u32,
            0,
            unwrap_win32_handle(parent.window_handle()),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
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

    pub async fn wait_click(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await;
            if msg.lParam == (self.handle.as_raw_window() as _)
                && ((msg.wParam as u32 >> 16) == BN_CLICKED)
            {
                break;
            }
        }
    }
}
