use std::{io, rc::Rc};

use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Controls::WC_EDITW,
        WindowsAndMessaging::{
            EN_UPDATE, ES_AUTOHSCROLL, ES_LEFT, WM_COMMAND, WS_CHILD, WS_EX_CLIENTEDGE, WS_TABSTOP,
            WS_VISIBLE,
        },
    },
};

use crate::ui::{AsRawWindow, Point, Size, Widget};

#[derive(Debug)]
pub struct TextBox {
    handle: Widget,
}

impl TextBox {
    pub fn new(parent: impl AsRawWindow) -> io::Result<Rc<Self>> {
        let handle = Widget::new(
            WC_EDITW,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_LEFT as u32 | ES_AUTOHSCROLL as u32,
            WS_EX_CLIENTEDGE,
            parent.as_raw_window(),
        )?;
        handle.set_size(handle.size_d2l((100, 50))).unwrap();
        Ok(Rc::new(Self { handle }))
    }

    pub fn loc(&self) -> io::Result<Point> {
        self.handle.loc()
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> io::Result<Size> {
        self.handle.size()
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)
    }

    pub fn text(&self) -> io::Result<String> {
        self.handle.text()
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.handle.set_text(s)
    }

    pub async fn wait_change(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await;
            if msg.lParam == self.as_raw_window() && ((msg.wParam as u32 >> 16) == EN_UPDATE) {
                break;
            }
        }
    }
}

impl AsRawWindow for TextBox {
    fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }
}
