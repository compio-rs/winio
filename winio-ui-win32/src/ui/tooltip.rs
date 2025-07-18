use std::ops::{Deref, DerefMut};

use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::{
        TOOLTIPS_CLASSW, TTF_IDISHWND, TTF_SUBCLASS, TTM_ADDTOOLW, TTM_DELTOOLW,
        TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW,
    },
    WindowsAndMessaging::{DestroyWindow, GetParent, WS_POPUP},
};
use winio_handle::{AsRawWidget, AsWidget};

use crate::Widget;

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
        self.text.to_string_lossy()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let add_new = self.text.is_empty();
        self.text = U16CString::from_str_truncate(s);
        if self.text.is_empty() {
            self.delete();
        } else {
            self.info.uId = self.inner.as_widget().as_win32() as _;
            self.info.lpszText = self.text.as_mut_ptr();
            if add_new {
                self.handle
                    .send_message(TTM_ADDTOOLW, 0, std::ptr::addr_of!(self.info) as _);
            } else {
                self.handle
                    .send_message(TTM_UPDATETIPTEXTW, 0, std::ptr::addr_of!(self.info) as _);
            }
        }
    }

    fn delete(&self) {
        self.handle
            .send_message(TTM_DELTOOLW, 0, std::ptr::addr_of!(self.info) as _);
    }
}

impl<T: AsWidget> Drop for ToolTip<T> {
    fn drop(&mut self) {
        unsafe {
            self.delete();
            DestroyWindow(self.handle.as_raw_widget().as_win32());
        }
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
