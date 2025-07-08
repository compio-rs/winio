use inherit_methods_macro::inherit_methods;
use widestring::U16CString;
use windows_sys::Win32::{
    Graphics::Gdi::InvalidateRect,
    UI::{
        Controls::WC_COMBOBOXW,
        WindowsAndMessaging::{
            CB_DELETESTRING, CB_GETCOUNT, CB_GETCURSEL, CB_GETLBTEXT, CB_GETLBTEXTLEN,
            CB_INSERTSTRING, CB_RESETCONTENT, CB_SETCURSEL, CBN_EDITUPDATE, CBN_SELCHANGE,
            CBS_AUTOHSCROLL, CBS_DROPDOWN, CBS_DROPDOWNLIST, CBS_HASSTRINGS, WM_COMMAND, WS_CHILD,
            WS_TABSTOP, WS_VISIBLE,
        },
    },
};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{Point, Size};

use crate::{
    runtime::WindowMessageCommand,
    ui::{Widget, font::measure_string, get_u16c, with_u16c},
};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl<const E: bool> ComboBoxImpl<E> {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut style =
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | CBS_AUTOHSCROLL as u32 | CBS_HASSTRINGS as u32;
        if E {
            style |= CBS_DROPDOWN as u32;
        } else {
            style |= CBS_DROPDOWNLIST as u32;
        }
        let mut handle = Widget::new(WC_COMBOBOXW, style, 0, parent.as_window().as_win32());
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let mut width = 0.0f64;
        for i in 0..self.len() {
            let data = self.get_u16(i);
            let s = measure_string(self.handle.as_raw_window().as_win32(), &data);
            width = width.max(s.width);
        }
        Size::new(width + 20.0, self.handle.size().height)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn selection(&self) -> Option<usize> {
        let i = self.handle.send_message(CB_GETCURSEL, 0, 0);
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = if let Some(i) = i { i as isize } else { -1 };
        self.handle.send_message(CB_SETCURSEL, i as _, 0);
    }

    pub async fn wait_select(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window().as_win32())
                && (message == CBN_SELCHANGE)
            {
                break;
            }
        }
    }

    pub async fn wait_change(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window().as_win32())
                && (message == CBN_EDITUPDATE)
            {
                break;
            }
        }
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        with_u16c(s.as_ref(), |s| {
            self.handle
                .send_message(CB_INSERTSTRING, i as _, s.as_ptr() as _);
        });
    }

    pub fn remove(&mut self, i: usize) {
        self.handle.send_message(CB_DELETESTRING, i as _, 0);
        unsafe {
            InvalidateRect(self.handle.as_raw_window().as_win32(), std::ptr::null(), 1);
        }
    }

    fn get_u16(&self, i: usize) -> U16CString {
        let len = self.handle.send_message(CB_GETLBTEXTLEN, i as _, 0);
        unsafe {
            get_u16c(len as usize, |buf| {
                self.handle
                    .send_message(CB_GETLBTEXT, i as _, buf.as_mut_ptr() as _) as _
            })
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
        self.handle.send_message(CB_GETCOUNT, 0, 0) as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.handle.send_message(CB_RESETCONTENT, 0, 0);
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;
