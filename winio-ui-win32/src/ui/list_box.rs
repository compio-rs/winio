use inherit_methods_macro::inherit_methods;
use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::WC_LISTBOXW,
    WindowsAndMessaging::{
        LB_DELETESTRING, LB_GETCOUNT, LB_GETSEL, LB_GETTEXT, LB_GETTEXTLEN, LB_INSERTSTRING,
        LB_RESETCONTENT, LB_SETSEL, LBN_SELCANCEL, LBN_SELCHANGE, LBS_DISABLENOSCROLL,
        LBS_HASSTRINGS, LBS_MULTIPLESEL, LBS_NOINTEGRALHEIGHT, LBS_NOTIFY, LBS_USETABSTOPS,
        WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
    },
};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{Point, Size};

use crate::{
    runtime::WindowMessageCommand,
    ui::{Widget, font::measure_string},
};

#[derive(Debug)]
pub struct ListBox {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            WC_LISTBOXW,
            WS_TABSTOP
                | WS_VISIBLE
                | WS_CHILD
                | WS_VSCROLL
                | LBS_NOTIFY as u32
                | LBS_MULTIPLESEL as u32
                | LBS_HASSTRINGS as u32
                | LBS_USETABSTOPS as u32
                | LBS_DISABLENOSCROLL as u32
                | LBS_NOINTEGRALHEIGHT as u32,
            0,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for i in 0..self.len() {
            let data = self.get_u16(i);
            let s = measure_string(self.handle.as_raw_window(), &data);
            width = width.max(s.width);
            height += s.height;
        }
        Size::new(width + 20.0, height)
    }

    pub fn min_size(&self) -> Size {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for i in 0..self.len() {
            let data = self.get_u16(i);
            let s = measure_string(self.handle.as_raw_window(), &data);
            width = width.max(s.width);
            height = height.max(s.height);
        }
        Size::new(width + 20.0, height)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn is_selected(&self, i: usize) -> bool {
        self.handle.send_message(LB_GETSEL, i as _, 0) != 0
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        self.handle
            .send_message(LB_SETSEL, if v { 1 } else { 0 }, i as _);
    }

    pub async fn wait_select(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window())
                && (message == LBN_SELCHANGE || message == LBN_SELCANCEL)
            {
                break;
            }
        }
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let s = U16CString::from_str_truncate(s);
        self.handle
            .send_message(LB_INSERTSTRING, i as _, s.as_ptr() as _);
    }

    pub fn remove(&mut self, i: usize) {
        self.handle.send_message(LB_DELETESTRING, i as _, 0);
    }

    fn get_u16(&self, i: usize) -> U16CString {
        let mut len = self.handle.send_message(LB_GETTEXTLEN, i as _, 0);
        if len == 0 {
            return U16CString::new();
        }
        len += 1;
        let mut res: Vec<u16> = Vec::with_capacity(len as usize);
        let len = self
            .handle
            .send_message(LB_GETTEXT, i as _, res.as_mut_ptr() as _);
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
        self.handle.send_message(LB_GETCOUNT, 0, 0) as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.handle.send_message(LB_RESETCONTENT, 0, 0);
    }
}
