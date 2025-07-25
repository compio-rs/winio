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
use winio_handle::{AsRawWidget, AsRawWindow, AsWindow, RawWidget};
use winio_primitive::{Point, Size};

use crate::{
    runtime::WindowMessageCommand,
    ui::{Widget, get_u16c, with_u16c},
};

#[derive(Debug)]
struct ComboBoxImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ComboBoxImpl {
    pub fn new(parent: impl AsWindow, editable: bool) -> Self {
        let mut style = WS_TABSTOP | WS_CHILD | CBS_AUTOHSCROLL as u32 | CBS_HASSTRINGS as u32;
        if editable {
            style |= CBS_DROPDOWN as u32;
        } else {
            style |= WS_VISIBLE | CBS_DROPDOWNLIST as u32;
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
            let s = self.handle.measure(data.as_ustr());
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

impl AsRawWidget for ComboBoxImpl {
    fn as_raw_widget(&self) -> RawWidget {
        self.handle.as_raw_widget()
    }
}

#[derive(Debug)]
pub struct ComboBox {
    handle: ComboBoxImpl,
    ehandle: ComboBoxImpl,
    editable: bool,
}

impl ComboBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let parent = parent.as_window();
        let handle = ComboBoxImpl::new(&parent, false);
        let ehandle = ComboBoxImpl::new(&parent, true);
        Self {
            handle,
            ehandle,
            editable: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        if self.editable {
            &self.ehandle
        } else {
            &self.handle
        }
        .is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        if self.editable {
            &mut self.ehandle
        } else {
            &mut self.handle
        }
        .set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
        self.ehandle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
        self.ehandle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        self.ehandle.set_size(v);
    }

    pub fn text(&self) -> String {
        if self.editable {
            &self.ehandle
        } else {
            &self.handle
        }
        .text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        if self.editable {
            &mut self.ehandle
        } else {
            &mut self.handle
        }
        .set_text(s);
    }

    pub fn selection(&self) -> Option<usize> {
        if self.editable {
            &self.ehandle
        } else {
            &self.handle
        }
        .selection()
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        if self.editable {
            &mut self.ehandle
        } else {
            &mut self.handle
        }
        .set_selection(i);
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn set_editable(&mut self, v: bool) {
        if self.editable != v {
            if v {
                self.ehandle.set_text(self.handle.text());
                self.ehandle.set_selection(self.handle.selection());
                self.ehandle.set_visible(self.handle.is_visible());
                self.handle.set_visible(false);
            } else {
                self.handle.set_text(self.ehandle.text());
                self.handle.set_selection(self.ehandle.selection());
                self.handle.set_visible(self.ehandle.is_visible());
                self.ehandle.set_visible(false);
            }
            self.editable = v;
        }
    }

    pub async fn wait_select(&self) {
        if self.editable {
            &self.ehandle
        } else {
            &self.handle
        }
        .wait_select()
        .await
    }

    pub async fn wait_change(&self) {
        if self.editable {
            &self.ehandle
        } else {
            &self.handle
        }
        .wait_change()
        .await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.handle.insert(i, s);
        self.ehandle.insert(i, s);
    }

    pub fn remove(&mut self, i: usize) {
        self.handle.remove(i);
        self.ehandle.remove(i);
    }

    pub fn get(&self, i: usize) -> String {
        self.handle.get(i)
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.handle.set(i, s);
        self.ehandle.set(i, s);
    }

    pub fn len(&self) -> usize {
        self.handle.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handle.is_empty()
    }

    pub fn clear(&mut self) {
        self.handle.clear();
        self.ehandle.clear();
    }
}

impl AsRawWidget for ComboBox {
    fn as_raw_widget(&self) -> RawWidget {
        if self.editable {
            &self.ehandle
        } else {
            &self.handle
        }
        .as_raw_widget()
    }

    fn iter_raw_widgets(&self) -> impl Iterator<Item = RawWidget> {
        [self.handle.as_raw_widget(), self.ehandle.as_raw_widget()].into_iter()
    }
}

winio_handle::impl_as_widget!(ComboBox);
