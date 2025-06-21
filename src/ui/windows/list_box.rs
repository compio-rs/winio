use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::{
        LVIF_TEXT, LVIS_SELECTED, LVITEMW, LVM_DELETEALLITEMS, LVM_DELETEITEM, LVM_GETITEMCOUNT,
        LVM_GETITEMSTATE, LVM_GETITEMTEXTW, LVM_INSERTITEMW, LVM_SETITEMSTATE, LVS_LIST, NM_CLICK,
        WC_LISTVIEWW,
    },
    WindowsAndMessaging::{
        SendMessageW, WM_NOTIFY, WS_CHILD, WS_HSCROLL, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
    },
};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    runtime::WindowMessageDetail,
    ui::{Widget, font::measure_string},
};

#[derive(Debug)]
pub struct ListBox {
    handle: Widget,
}

impl ListBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            WC_LISTVIEWW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | WS_HSCROLL | WS_VSCROLL | LVS_LIST,
            0,
            parent.as_window().as_raw_window(),
        );
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

    pub fn is_selected(&self, i: usize) -> bool {
        let status = unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                LVM_GETITEMSTATE,
                i as _,
                LVIS_SELECTED as _,
            )
        };
        status == LVIS_SELECTED as _
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        unsafe {
            let mut item: LVITEMW = std::mem::zeroed();
            item.stateMask = LVIS_SELECTED;
            item.state = if v { LVIS_SELECTED } else { 0 };
            SendMessageW(
                self.handle.as_raw_window(),
                LVM_SETITEMSTATE,
                i as _,
                std::ptr::addr_of!(item) as _,
            );
        }
    }

    pub async fn wait_select(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_NOTIFY).await;
            if let Some(WindowMessageDetail::Command {
                message, handle, ..
            }) = msg.detail
            {
                if std::ptr::eq(handle, self.handle.as_raw_window()) && (message == NM_CLICK) {
                    break;
                }
            }
        }
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let mut s = U16CString::from_str_truncate(s);
        unsafe {
            let mut item: LVITEMW = std::mem::zeroed();
            item.mask = LVIF_TEXT;
            item.iItem = i as _;
            item.pszText = s.as_mut_ptr();
            SendMessageW(
                self.handle.as_raw_window(),
                LVM_INSERTITEMW,
                0,
                std::ptr::addr_of!(item) as _,
            )
        };
    }

    pub fn remove(&mut self, i: usize) {
        unsafe {
            SendMessageW(self.handle.as_raw_window(), LVM_DELETEITEM, i as _, 0);
        }
    }

    fn get_u16(&self, i: usize) -> U16CString {
        let mut res: Vec<u16> = Vec::with_capacity(256);
        let mut len;
        unsafe {
            loop {
                let mut item: LVITEMW = std::mem::zeroed();
                item.pszText = res.as_mut_ptr();
                item.cchTextMax = res.len() as _;
                len = SendMessageW(
                    self.handle.as_raw_window(),
                    LVM_GETITEMTEXTW,
                    i as _,
                    std::ptr::addr_of!(item) as _,
                ) as usize;
                if len < res.len() - 1 {
                    break;
                } else {
                    res.reserve(res.capacity());
                }
            }
            res.set_len(len + 1);
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
        unsafe { SendMessageW(self.handle.as_raw_window(), LVM_GETITEMCOUNT, 0, 0) as _ }
    }

    pub fn clear(&mut self) {
        unsafe { SendMessageW(self.handle.as_raw_window(), LVM_DELETEALLITEMS, 0, 0) };
    }
}
