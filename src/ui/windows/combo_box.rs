use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::WC_COMBOBOXW,
    WindowsAndMessaging::{
        CB_DELETESTRING, CB_GETCOUNT, CB_GETCURSEL, CB_GETLBTEXT, CB_GETLBTEXTLEN, CB_INSERTSTRING,
        CB_RESETCONTENT, CB_SETCURSEL, CBN_EDITUPDATE, CBN_SELCHANGE, CBS_AUTOHSCROLL,
        CBS_DROPDOWN, CBS_DROPDOWNLIST, CBS_HASSTRINGS, SendMessageW, WM_COMMAND, WS_CHILD,
        WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    runtime::WindowMessageDetail,
    ui::{Widget, font::measure_string},
};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    handle: Widget,
}

impl<const E: bool> ComboBoxImpl<E> {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut style =
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | CBS_AUTOHSCROLL as u32 | CBS_HASSTRINGS as u32;
        if E {
            style |= CBS_DROPDOWN as u32;
        } else {
            style |= CBS_DROPDOWNLIST as u32;
        }
        let mut handle = Widget::new(WC_COMBOBOXW, style, 0, parent.as_window().as_raw_window());
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        let mut width = 0.0f64;
        for i in 0..self.len() {
            let data = self.get_u16(i);
            let s = measure_string(self.handle.as_raw_window(), &data);
            width = width.max(s.width);
        }
        Size::new(width + 20.0, self.handle.size().height)
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

    pub fn selection(&self) -> Option<usize> {
        let i = unsafe { SendMessageW(self.handle.as_raw_window(), CB_GETCURSEL, 0, 0) };
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = if let Some(i) = i { i as isize } else { -1 };
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_SETCURSEL, i as _, 0) };
    }

    pub async fn wait_select(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await;
            if let Some(WindowMessageDetail::Command {
                message, handle, ..
            }) = msg.detail
            {
                if std::ptr::eq(handle, self.handle.as_raw_window()) && (message == CBN_SELCHANGE) {
                    break;
                }
            }
        }
    }

    pub async fn wait_change(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await;
            if let Some(WindowMessageDetail::Command {
                message, handle, ..
            }) = msg.detail
            {
                if std::ptr::eq(handle, self.handle.as_raw_window()) && (message == CBN_EDITUPDATE)
                {
                    break;
                }
            }
        }
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let s = U16CString::from_str_truncate(s);
        unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                CB_INSERTSTRING,
                i as _,
                s.as_ptr() as _,
            )
        };
    }

    pub fn remove(&mut self, i: usize) {
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_DELETESTRING, i as _, 0) };
    }

    fn get_u16(&self, i: usize) -> U16CString {
        let mut len =
            unsafe { SendMessageW(self.handle.as_raw_window(), CB_GETLBTEXTLEN, i as _, 0) };
        if len == 0 {
            return U16CString::new();
        }
        len += 1;
        let mut res: Vec<u16> = Vec::with_capacity(len as usize);
        let len = unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                CB_GETLBTEXT,
                i as _,
                res.as_mut_ptr() as _,
            )
        };
        unsafe {
            res.set_len(len as usize + 1);
            U16CString::from_vec_unchecked(res)
        }
    }

    pub fn get(&self, i: usize) -> String {
        self.get_u16(i).to_string_lossy()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        self.remove(i);
        self.insert(i, s);
    }

    pub fn len(&self) -> usize {
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_GETCOUNT, 0, 0) as _ }
    }

    pub fn clear(&mut self) {
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_RESETCONTENT, 0, 0) };
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;
