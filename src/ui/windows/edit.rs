use std::ptr::null;

use windows_sys::Win32::{
    Graphics::Gdi::InvalidateRect,
    UI::{
        Controls::{EM_GETPASSWORDCHAR, EM_SETPASSWORDCHAR, ShowScrollBar, WC_EDITW},
        WindowsAndMessaging::{
            EN_UPDATE, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_CENTER, ES_LEFT, ES_MULTILINE,
            ES_PASSWORD, ES_RIGHT, SB_VERT, SendMessageW, WM_COMMAND, WS_CHILD, WS_EX_CLIENTEDGE,
            WS_TABSTOP, WS_VISIBLE,
        },
    },
};

use crate::{
    AsRawWindow, AsWindow, HAlign, Point, RawWindow, Size,
    runtime::WindowMessageDetail,
    ui::{Widget, font::measure_string, sys::fix_crlf},
};

#[derive(Debug)]
struct EditImpl {
    handle: Widget,
}

impl EditImpl {
    pub fn new(parent: impl AsWindow, style: u32) -> Self {
        let handle = Widget::new(
            WC_EDITW,
            style,
            WS_EX_CLIENTEDGE,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((100, 50)));
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
        Size::new(s.width + 8.0, s.height + 4.0)
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
            HAlign::Center => style |= ES_CENTER as u32,
            HAlign::Right => style |= ES_RIGHT as u32,
            _ => style |= ES_LEFT as u32,
        }
        self.handle.set_style(style)
    }

    pub async fn wait_change(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await;
            if let Some(WindowMessageDetail::Command {
                message, handle, ..
            }) = msg.detail
            {
                if std::ptr::eq(handle, self.handle.as_raw_window()) && (message == EN_UPDATE) {
                    break;
                }
            }
        }
    }
}

impl AsRawWindow for EditImpl {
    fn as_raw_window(&self) -> RawWindow {
        self.handle.as_raw_window()
    }
}

#[derive(Debug)]
pub struct Edit {
    handle: EditImpl,
    pchar: u16,
}

impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = EditImpl::new(
            parent,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | ES_LEFT as u32
                | ES_AUTOHSCROLL as u32
                | ES_PASSWORD as u32,
        );
        let mut pchar =
            unsafe { SendMessageW(handle.as_raw_window(), EM_GETPASSWORDCHAR, 0, 0) } as u16;
        if pchar == 0 {
            pchar = '*' as u32 as _;
        }
        unsafe { SendMessageW(handle.as_raw_window(), EM_SETPASSWORDCHAR, 0, 0) };
        Self { handle, pchar }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
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

    pub fn is_password(&self) -> bool {
        unsafe { SendMessageW(self.handle.as_raw_window(), EM_GETPASSWORDCHAR, 0, 0) != 0 }
    }

    pub fn set_password(&mut self, v: bool) {
        unsafe {
            if v {
                SendMessageW(
                    self.handle.as_raw_window(),
                    EM_SETPASSWORDCHAR,
                    self.pchar as _,
                    0,
                );
            } else {
                SendMessageW(self.handle.as_raw_window(), EM_SETPASSWORDCHAR, 0, 0);
            }
        }
        unsafe {
            InvalidateRect(self.handle.as_raw_window(), null(), 1);
        }
    }

    pub fn halign(&self) -> HAlign {
        self.handle.halign()
    }

    pub fn set_halign(&mut self, align: HAlign) {
        self.handle.set_halign(align);
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

#[derive(Debug)]
pub struct TextBox {
    handle: EditImpl,
}

impl TextBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = EditImpl::new(
            parent,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | ES_LEFT as u32
                | ES_MULTILINE as u32
                | ES_AUTOVSCROLL as u32,
        );
        unsafe { ShowScrollBar(handle.as_raw_window(), SB_VERT, 1) };
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
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
        self.handle.text().replace("\r\n", "\n")
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(fix_crlf(s.as_ref()))
    }

    pub fn halign(&self) -> HAlign {
        self.handle.halign()
    }

    pub fn set_halign(&mut self, align: HAlign) {
        self.handle.set_halign(align);
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}
