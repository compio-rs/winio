use windows_sys::Win32::UI::{
    Controls::WC_EDITW,
    WindowsAndMessaging::{
        EN_UPDATE, ES_AUTOHSCROLL, ES_CENTER, ES_LEFT, ES_RIGHT, WM_COMMAND, WS_CHILD,
        WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{AsRawWindow, AsWindow, HAlign, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Edit {
    handle: Widget,
}

impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = Widget::new(
            WC_EDITW,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_LEFT as u32 | ES_AUTOHSCROLL as u32,
            WS_EX_CLIENTEDGE,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((100, 50)));
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

    pub fn halign(&self) -> HAlign {
        let style = self.handle.style() as i32;
        if (style & ES_RIGHT) == ES_RIGHT {
            HAlign::Right
        } else if (style & ES_CENTER) == ES_CENTER {
            HAlign::Center
        } else {
            HAlign::Left
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        let mut style = self.handle.style();
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
            if msg.lParam == (self.handle.as_raw_window() as _)
                && ((msg.wParam as u32 >> 16) == EN_UPDATE)
            {
                break;
            }
        }
    }
}
