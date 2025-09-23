use std::ptr::null;

use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::InvalidateRect,
        UI::{
            Controls::{
                EM_GETPASSWORDCHAR, EM_REPLACESEL, EM_SETPASSWORDCHAR, ShowScrollBar, WC_EDITW,
            },
            Input::KeyboardAndMouse::VK_RETURN,
            Shell::{DefSubclassProc, SetWindowSubclass},
            WindowsAndMessaging::{
                DLGC_WANTALLKEYS, EN_UPDATE, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_CENTER, ES_LEFT,
                ES_MULTILINE, ES_PASSWORD, ES_RIGHT, SB_VERT, WM_COMMAND, WM_GETDLGCODE, WM_KEYUP,
                WS_CHILD, WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    w,
};
use winio_handle::{AsContainer, AsRawWidget, AsRawWindow};
use winio_primitive::{HAlign, Point, Size};

use crate::{
    runtime::WindowMessageCommand,
    ui::{Widget, fix_crlf},
};

#[derive(Debug)]
struct EditImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl EditImpl {
    pub fn new(parent: impl AsContainer, style: u32) -> Self {
        let mut handle = Widget::new(
            WC_EDITW,
            style,
            WS_EX_CLIENTEDGE,
            parent.as_container().as_win32(),
        );
        handle.set_size(handle.size_d2l((100, 50)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let s = self.handle.measure_text();
        Size::new(s.width + 8.0, s.height + 4.0)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

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
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window().as_win32())
                && (message == EN_UPDATE)
            {
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(EditImpl, handle);

#[derive(Debug)]
pub struct Edit {
    handle: EditImpl,
    pchar: u16,
}

#[inherit_methods(from = "self.handle")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = EditImpl::new(
            parent,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | ES_LEFT as u32
                | ES_AUTOHSCROLL as u32
                | ES_PASSWORD as u32,
        );
        let mut pchar = handle.handle.send_message(EM_GETPASSWORDCHAR, 0, 0) as u16;
        if pchar == 0 {
            pchar = '*' as u16;
        }
        handle.handle.send_message(EM_SETPASSWORDCHAR, 0, 0);
        Self { handle, pchar }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn halign(&self) -> HAlign;

    pub fn set_halign(&mut self, align: HAlign);

    pub fn is_password(&self) -> bool {
        self.handle.handle.send_message(EM_GETPASSWORDCHAR, 0, 0) != 0
    }

    pub fn set_password(&mut self, v: bool) {
        if v {
            self.handle
                .handle
                .send_message(EM_SETPASSWORDCHAR, self.pchar as _, 0);
        } else {
            self.handle.handle.send_message(EM_SETPASSWORDCHAR, 0, 0);
        }
        unsafe {
            InvalidateRect(self.handle.as_raw_widget().as_win32(), null(), 1);
        }
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(Edit, handle);

#[derive(Debug)]
pub struct TextBox {
    handle: EditImpl,
}

#[inherit_methods(from = "self.handle")]
impl TextBox {
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = EditImpl::new(
            parent,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | ES_LEFT as u32
                | ES_MULTILINE as u32
                | ES_AUTOVSCROLL as u32,
        );
        unsafe { ShowScrollBar(handle.as_raw_widget().as_win32(), SB_VERT, 1) };
        syscall!(
            BOOL,
            SetWindowSubclass(
                handle.as_raw_widget().as_win32(),
                Some(multiline_edit_wnd_proc),
                0,
                0,
            )
        )
        .unwrap();
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size {
        let text = self.handle.handle.text_u16();
        let index = text.as_slice().iter().position(|c| *c == '\r' as u16);
        if let Some(index) = index {
            let s = self.handle.handle.measure(text.split_at(index).0);
            Size::new(8.0, s.height + 4.0)
        } else {
            self.preferred_size()
        }
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn text(&self) -> String {
        self.handle.text().replace("\r\n", "\n")
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(fix_crlf(s.as_ref()))
    }

    pub fn halign(&self) -> HAlign;

    pub fn set_halign(&mut self, align: HAlign);

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(TextBox, handle);

unsafe extern "system" fn multiline_edit_wnd_proc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    let mut res = DefSubclassProc(hwnd, umsg, wparam, lparam);
    match umsg {
        WM_GETDLGCODE => {
            res &= !(DLGC_WANTALLKEYS as isize);
        }
        WM_KEYUP => {
            if wparam == VK_RETURN as _ {
                const RETURN_TEXT: *const u16 = w!("\r\n");
                DefSubclassProc(hwnd, EM_REPLACESEL, 1, RETURN_TEXT as _);
            }
        }
        _ => {}
    }
    res
}
