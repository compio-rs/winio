use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::WC_COMBOBOXW,
    WindowsAndMessaging::{
        CB_DELETESTRING, CB_GETCOUNT, CB_GETCURSEL, CB_GETITEMDATA, CB_INSERTSTRING,
        CB_RESETCONTENT, CB_SETCURSEL, CB_SETITEMDATA, CBN_SELCHANGE, CBS_AUTOHSCROLL,
        CBS_DROPDOWN, CBS_DROPDOWNLIST, CBS_HASSTRINGS, SendMessageW, WM_COMMAND, WS_CHILD,
        WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{AsRawWindow, AsWindow, Point, Size, ui::Widget};

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
        let handle = Widget::new(WC_COMBOBOXW, style, 0, parent.as_window().as_raw_window());
        handle.set_size(handle.size_d2l((50, 14)));
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

    pub fn selection(&self) -> Option<usize> {
        let i = unsafe { SendMessageW(self.handle.as_raw_window(), CB_GETCURSEL, 0, 0) };
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = if let Some(i) = i { i as isize } else { -1 };
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_SETCURSEL, i as _, 0) };
    }

    pub async fn wait_change(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await;
            if msg.lParam == (self.handle.as_raw_window() as _)
                && ((msg.wParam as u32 >> 16) == CBN_SELCHANGE)
            {
                break;
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

    pub fn get(&self, i: usize) -> String {
        let data = unsafe { SendMessageW(self.handle.as_raw_window(), CB_GETITEMDATA, i as _, 0) };
        unsafe { U16CString::from_ptr_str(data as _) }.to_string_lossy()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        let s = U16CString::from_str_truncate(s);
        unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                CB_SETITEMDATA,
                i as _,
                s.as_ptr() as _,
            )
        };
    }

    pub fn len(&self) -> usize {
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_GETCOUNT, 0, 0) as _ }
    }

    pub fn clear(&mut self) {
        unsafe { SendMessageW(self.handle.as_raw_window(), CB_RESETCONTENT, 0, 0) };
    }
}

pub type ComboBox = ComboBoxImpl<false>;
