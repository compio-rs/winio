use inherit_methods_macro::inherit_methods;
use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::WC_LISTBOXW,
    WindowsAndMessaging::{
        GetParent, LB_DELETESTRING, LB_GETCOUNT, LB_GETSEL, LB_GETTEXT, LB_GETTEXTLEN,
        LB_INSERTSTRING, LB_RESETCONTENT, LB_SETSEL, LBN_SELCANCEL, LBN_SELCHANGE,
        LBS_DISABLENOSCROLL, LBS_HASSTRINGS, LBS_MULTIPLESEL, LBS_NOINTEGRALHEIGHT, LBS_NOTIFY,
        LBS_USETABSTOPS, WM_COMMAND, WS_CHILD, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
    },
};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{
    Result,
    runtime::WindowMessageCommand,
    ui::{Widget, get_u16c, with_u16c},
};

#[derive(Debug)]
struct ListBoxImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ListBoxImpl {
    pub fn new(parent: impl AsContainer, multiple: bool) -> Result<Self> {
        let mut style = WS_TABSTOP
            | WS_VISIBLE
            | WS_CHILD
            | WS_VSCROLL
            | LBS_NOTIFY as u32
            | LBS_HASSTRINGS as u32
            | LBS_USETABSTOPS as u32
            | LBS_DISABLENOSCROLL as u32
            | LBS_NOINTEGRALHEIGHT as u32;
        if multiple {
            style |= LBS_MULTIPLESEL as u32;
        }
        let handle = Widget::new(WC_LISTBOXW, style, 0, parent.as_container().as_win32())?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
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

    pub fn min_size(&self) -> Result<Size> {
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

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        Ok(self.handle.send_message(LB_GETSEL, i as _, 0) != 0)
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        self.handle
            .send_message(LB_SETSEL, if v { 1 } else { 0 }, i as _);
        Ok(())
    }

    pub async fn wait_select(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_widget().as_win32())
                && (message == LBN_SELCHANGE || message == LBN_SELCANCEL)
            {
                break;
            }
        }
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        with_u16c(s.as_ref(), |s| {
            self.handle
                .send_message(LB_INSERTSTRING, i as _, s.as_ptr() as _);
            Ok(())
        })
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.handle.send_message(LB_DELETESTRING, i as _, 0);
        Ok(())
    }

    fn get_u16(&self, i: usize) -> Result<U16CString> {
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

    pub fn get(&self, i: usize) -> Result<String> {
        Ok(self.get_u16(i)?.to_string_lossy())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.remove(i)?;
        self.insert(i, s)
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.handle.send_message(LB_GETCOUNT, 0, 0) as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.handle.send_message(LB_RESETCONTENT, 0, 0);
        Ok(())
    }
}

winio_handle::impl_as_widget!(ListBoxImpl, handle);

#[derive(Debug)]
pub struct ListBox {
    handle: ListBoxImpl,
    multiple: bool,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = ListBoxImpl::new(&parent, false)?;
        Ok(Self {
            handle,
            multiple: false,
        })
    }

    fn recreate(&mut self, multiple: bool) -> Result<()> {
        let parent = unsafe { GetParent(self.handle.as_widget().as_win32()) };
        let mut new_handle =
            ListBoxImpl::new(unsafe { BorrowedContainer::win32(parent) }, multiple)?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_tooltip(self.handle.tooltip()?)?;
        for i in 0..self.handle.len()? {
            new_handle.insert(i, self.handle.get(i)?)?;
        }
        for i in 0..self.handle.len()? {
            new_handle.set_selected(i, self.handle.is_selected(i)?)?;
        }
        self.handle = new_handle;
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_multiple(&self) -> Result<bool> {
        Ok(self.multiple)
    }

    pub fn set_multiple(&mut self, v: bool) -> Result<()> {
        if self.multiple != v {
            self.recreate(v)?;
            self.multiple = v;
        }
        Ok(())
    }

    pub fn is_selected(&self, i: usize) -> Result<bool>;

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()>;

    pub async fn wait_select(&self) {
        self.handle.wait_select().await;
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()>;

    pub fn remove(&mut self, i: usize) -> Result<()>;

    pub fn get(&self, i: usize) -> Result<String>;

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()>;

    pub fn len(&self) -> Result<usize>;

    pub fn is_empty(&self) -> Result<bool>;

    pub fn clear(&mut self) -> Result<()>;
}

winio_handle::impl_as_widget!(ListBox, handle);
