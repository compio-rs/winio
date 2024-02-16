use std::{io, rc::Rc};

use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Controls::WC_EDITW,
        WindowsAndMessaging::{
            EN_UPDATE, ES_AUTOHSCROLL, ES_CENTER, ES_LEFT, ES_RIGHT, WM_COMMAND, WS_CHILD,
            WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
        },
    },
};

use crate::ui::{AsRawWindow, HAlign, Point, Size, Widget};

#[derive(Debug)]
pub struct Edit {
    handle: Widget,
}

impl Edit {
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

    pub fn halign(&self) -> io::Result<HAlign> {
        let style = self.handle.style()? as i32;
        if (style & ES_RIGHT) == ES_RIGHT {
            Ok(HAlign::Right)
        } else if (style & ES_CENTER) == ES_CENTER {
            Ok(HAlign::Center)
        } else {
            Ok(HAlign::Left)
        }
    }

    pub fn set_halign(&self, align: HAlign) -> io::Result<()> {
        let mut style = self.handle.style()?;
        style &= !(ES_RIGHT as u32);
        match align {
            HAlign::Left => style |= ES_LEFT as u32,
            HAlign::Center => style |= ES_CENTER as u32,
            HAlign::Right => style |= ES_RIGHT as u32,
        }
        self.handle.set_style(style)
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

impl AsRawWindow for Edit {
    fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }
}
