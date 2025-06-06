use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BN_CLICKED, BS_PUSHBUTTON, WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    runtime::WindowMessageDetail,
    ui::{Widget, font::measure_string},
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
}

impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON as u32,
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

    pub fn preferred_size(&self) -> Size {
        let s = measure_string(self.handle.as_raw_window(), &self.handle.text_u16());
        Size::new(s.width + 4.0, s.height + 4.0)
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
            if let Some(WindowMessageDetail::Command {
                message, handle, ..
            }) = msg.detail
            {
                if std::ptr::eq(handle, self.handle.as_raw_window()) && (message == BN_CLICKED) {
                    break;
                }
            }
        }
    }
}
