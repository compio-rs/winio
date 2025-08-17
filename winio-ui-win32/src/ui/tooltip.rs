use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::{
        TOOLTIPS_CLASSW, TTF_IDISHWND, TTF_SUBCLASS, TTM_ADDTOOLW, TTM_DELTOOLW,
        TTM_SETMAXTIPWIDTH, TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW,
    },
    WindowsAndMessaging::{GetParent, GetSystemMetrics, SM_CXMAXTRACK, WS_POPUP},
};
use winio_handle::AsWidget;

use crate::{Widget, ui::fix_crlf};

pub struct ToolTip<T: AsWidget> {
    inner: T,
    handle: Widget,
    info: TTTOOLINFOW,
    text: U16CString,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        let parent = unsafe { GetParent(inner.as_widget().as_win32()) };
        let handle = Widget::new(
            TOOLTIPS_CLASSW,
            WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP,
            0,
            parent,
        );

        // Enable support for multiline tooltips
        // -1 doesn't work, we use SM_CXMAXTRACK like WinForms does
        let max_width = unsafe { GetSystemMetrics(SM_CXMAXTRACK) };
        handle.send_message(TTM_SETMAXTIPWIDTH, 0, max_width as isize);

        let mut info: TTTOOLINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<TTTOOLINFOW>() as _;
        info.uFlags = TTF_SUBCLASS | TTF_IDISHWND;
        info.hwnd = parent;
        Self {
            inner,
            handle,
            info,
            text: U16CString::new(),
        }
    }

    pub fn tooltip(&self) -> String {
        self.text.to_string_lossy().replace("\r\n", "\n")
    }

    fn update_info(&mut self, msg: u32) {
        for handle in self.inner.iter_widgets() {
            self.info.uId = handle.as_win32() as _;
            self.handle
                .send_message(msg, 0, std::ptr::addr_of!(self.info) as _);
        }
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let add_new = self.text.is_empty();
        self.text = U16CString::from_str_truncate(fix_crlf(s.as_ref()));
        if self.text.is_empty() {
            self.delete();
        } else {
            self.info.lpszText = self.text.as_mut_ptr();
            let msg = if add_new {
                TTM_ADDTOOLW
            } else {
                TTM_UPDATETIPTEXTW
            };
            self.update_info(msg);
        }
    }

    fn delete(&mut self) {
        self.update_info(TTM_DELTOOLW);
    }
}

impl<T: AsWidget + Debug> Debug for ToolTip<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolTip")
            .field("inner", &self.inner)
            .field("handle", &self.handle)
            .field("text", &self.text)
            .finish()
    }
}

impl<T: AsWidget> Deref for ToolTip<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AsWidget> DerefMut for ToolTip<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
