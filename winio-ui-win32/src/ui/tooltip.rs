use std::{cell::RefCell, collections::BTreeMap};

use widestring::U16CString;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Controls::{
            TOOLTIPS_CLASSW, TTF_IDISHWND, TTF_SUBCLASS, TTM_ADDTOOLW, TTM_DELTOOLW,
            TTM_SETMAXTIPWIDTH, TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW,
        },
        WindowsAndMessaging::{GetParent, GetSystemMetrics, SM_CXMAXTRACK, WS_POPUP},
    },
};

use crate::{Widget, ui::fix_crlf};

thread_local! {
    static TOOLTIPS: RefCell<BTreeMap<HWND, ToolTip>> = const { RefCell::new(BTreeMap::new()) };
}

pub(crate) fn set_tooltip(hwnd: HWND, s: impl AsRef<str>) {
    let s = s.as_ref();
    if s.is_empty() {
        remove_tooltip(hwnd);
    } else {
        TOOLTIPS.with_borrow_mut(|m| {
            m.entry(hwnd)
                .or_insert_with(|| ToolTip::new(hwnd))
                .set_tooltip(s);
        });
    }
}

pub(crate) fn get_tooltip(hwnd: HWND) -> Option<String> {
    TOOLTIPS.with_borrow(|m| m.get(&hwnd).map(|t| t.tooltip()))
}

pub(crate) fn remove_tooltip(hwnd: HWND) {
    TOOLTIPS.with_borrow_mut(|m| m.remove(&hwnd));
}

pub(crate) struct ToolTip {
    handle: Widget,
    info: TTTOOLINFOW,
    text: U16CString,
}

impl ToolTip {
    pub fn new(hwnd: HWND) -> Self {
        let parent = unsafe { GetParent(hwnd) };
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
        info.uId = hwnd as _;
        Self {
            handle,
            info,
            text: U16CString::new(),
        }
    }

    pub fn tooltip(&self) -> String {
        self.text.to_string_lossy().replace("\r\n", "\n")
    }

    fn update_info(&mut self, msg: u32) {
        self.handle
            .send_message(msg, 0, std::ptr::addr_of!(self.info) as _);
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

impl Drop for ToolTip {
    fn drop(&mut self) {
        self.delete();
    }
}
