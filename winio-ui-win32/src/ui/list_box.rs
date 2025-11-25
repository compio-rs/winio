use std::io;

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
use winio_handle::{AsContainer, AsRawWindow};
use winio_primitive::{Point, Size};

use crate::{
    runtime::WindowMessageCommand,
    ui::{Widget, get_u16c, with_u16c},
};

#[derive(Debug)]
pub struct ListBox {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> io::Result<Self> {
        let handle = Widget::new(
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
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> io::Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> io::Result<()>;

    pub fn is_enabled(&self) -> io::Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> io::Result<()>;

    pub fn preferred_size(&self) -> io::Result<Size> {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for i in 0..self.len()? {
            let data = self.get_u16(i)?;
            let s = self.handle.measure(data.as_ustr())?;
            width = width.max(s.width);
            height += s.height;
        }
        Ok(Size::new(width + 20.0, height))
    }

    pub fn min_size(&self) -> io::Result<Size> {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for i in 0..self.len()? {
            let data = self.get_u16(i)?;
            let s = self.handle.measure(data.as_ustr())?;
            width = width.max(s.width);
            height = height.max(s.height);
        }
        Ok(Size::new(width + 20.0, height))
    }

    pub fn loc(&self) -> io::Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> io::Result<()>;

    pub fn size(&self) -> io::Result<Size>;

    pub fn set_size(&mut self, v: Size) -> io::Result<()>;

    pub fn tooltip(&self) -> io::Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> io::Result<()>;

    pub fn is_selected(&self, i: usize) -> io::Result<bool> {
        Ok(self.handle.send_message(LB_GETSEL, i as _, 0) != 0)
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> io::Result<()> {
        self.handle
            .send_message(LB_SETSEL, if v { 1 } else { 0 }, i as _);
        Ok(())
    }

    pub async fn wait_select(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_raw_window().as_win32())
                && (message == LBN_SELCHANGE || message == LBN_SELCANCEL)
            {
                break;
            }
        }
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> io::Result<()> {
        with_u16c(s.as_ref(), |s| {
            self.handle
                .send_message(LB_INSERTSTRING, i as _, s.as_ptr() as _);
            Ok(())
        })
    }

    pub fn remove(&mut self, i: usize) -> io::Result<()> {
        self.handle.send_message(LB_DELETESTRING, i as _, 0);
        Ok(())
    }

    fn get_u16(&self, i: usize) -> io::Result<U16CString> {
        let len = self.handle.send_message(LB_GETTEXTLEN, i as _, 0);
        unsafe {
            get_u16c(len as usize, |buf| {
                Ok(self
                    .handle
                    .send_message(LB_GETTEXT, i as _, buf.as_mut_ptr() as _)
                    as _)
            })
        }
    }

    pub fn get(&self, i: usize) -> io::Result<String> {
        Ok(self.get_u16(i)?.to_string_lossy())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> io::Result<()> {
        self.remove(i)?;
        self.insert(i, s)
    }

    pub fn len(&self) -> io::Result<usize> {
        Ok(self.handle.send_message(LB_GETCOUNT, 0, 0) as _)
    }

    pub fn is_empty(&self) -> io::Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.handle.send_message(LB_RESETCONTENT, 0, 0);
        Ok(())
    }
}

winio_handle::impl_as_widget!(ListBox, handle);
