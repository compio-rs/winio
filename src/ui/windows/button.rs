use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BN_CLICKED, BS_PUSHBUTTON, WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
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

    pub fn preferred_size(&self) -> Size {
        let s = measure_string(self.handle.as_raw_window(), &self.handle.text_u16());
        Size::new(s.width + 5.0, s.height + 2.0)
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
