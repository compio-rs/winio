use inherit_methods_macro::inherit_methods;
use widestring::U16CString;
use windows_sys::Win32::UI::{
    Controls::WC_COMBOBOXW,
    WindowsAndMessaging::{
        CB_DELETESTRING, CB_GETCOUNT, CB_GETCURSEL, CB_GETLBTEXT, CB_GETLBTEXTLEN, CB_INSERTSTRING,
        CB_RESETCONTENT, CB_SETCURSEL, CBN_EDITUPDATE, CBN_SELCHANGE, CBS_AUTOHSCROLL,
        CBS_DROPDOWN, CBS_DROPDOWNLIST, CBS_HASSTRINGS, GetParent, WM_COMMAND, WS_CHILD,
        WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{
    AsContainer, AsRawWidget, AsRawWindow, BorrowedContainer, RawContainer, RawWidget,
};
use winio_primitive::{Point, Size};

use crate::{
    Result,
    runtime::WindowMessageCommand,
    ui::{Widget, get_u16c, with_u16c},
};

#[derive(Debug)]
struct ComboBoxImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ComboBoxImpl {
    pub fn new(parent: impl AsContainer, editable: bool) -> Result<Self> {
        let mut style = WS_TABSTOP | WS_CHILD | CBS_AUTOHSCROLL as u32 | CBS_HASSTRINGS as u32;
        if editable {
            style |= CBS_DROPDOWN as u32;
        } else {
            style |= WS_VISIBLE | CBS_DROPDOWNLIST as u32;
        }
        let handle = Widget::new(WC_COMBOBOXW, style, 0, parent.as_container().as_win32())?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let mut width = 0.0f64;
        for i in 0..self.len()? {
            let data = self.get_u16(i)?;
            let s = self.handle.measure(data.as_ustr())?;
            width = width.max(s.width);
        }
        Ok(Size::new(width + 20.0, self.handle.size()?.height))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        let i = self.handle.send_message(CB_GETCURSEL, 0, 0);
        Ok(if i < 0 { None } else { Some(i as _) })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.handle.send_message(CB_SETCURSEL, i as _, 0);
        Ok(())
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

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        with_u16c(s.as_ref(), |s| {
            self.handle
                .send_message(CB_INSERTSTRING, i as _, s.as_ptr() as _);
            Ok(())
        })
    }

    pub fn remove(&mut self, i: usize, editable: bool) -> Result<()> {
        let remove_current = self.selection()? == Some(i);
        self.handle.send_message(CB_DELETESTRING, i as _, 0);
        let len = self.len()?;
        if remove_current && (!editable) {
            if len > 0 {
                self.set_selection(i.min(len - 1))?;
            } else {
                self.handle.send_message(CB_SETCURSEL, -1isize as _, 0);
            }
        }
        self.handle.invalidate(true)
    }

    fn get_u16(&self, i: usize) -> Result<U16CString> {
        let len = self.handle.send_message(CB_GETLBTEXTLEN, i as _, 0);
        unsafe {
            get_u16c(len as usize, |buf| {
                Ok(self
                    .handle
                    .send_message(CB_GETLBTEXT, i as _, buf.as_mut_ptr() as _)
                    as _)
            })
        }
    }

    pub fn get(&self, i: usize) -> Result<String> {
        Ok(self.get_u16(i)?.to_string_lossy())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        let selection = self.selection()?;
        self.remove(i, true)?;
        self.insert(i, s)?;
        if let Some(i) = selection {
            self.set_selection(i)?;
        }
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.handle.send_message(CB_GETCOUNT, 0, 0) as _)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.handle.send_message(CB_RESETCONTENT, 0, 0);
        Ok(())
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
    editable: bool,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = ComboBoxImpl::new(&parent, false)?;
        Ok(Self {
            handle,
            editable: false,
        })
    }

    fn recreate(&mut self, editable: bool) -> Result<()> {
        let parent = unsafe { GetParent(self.handle.as_raw_widget().as_win32()) };
        let mut new_handle = ComboBoxImpl::new(
            unsafe { BorrowedContainer::borrow_raw(RawContainer::Win32(parent)) },
            editable,
        )?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_tooltip(self.handle.tooltip()?)?;
        new_handle.set_text(self.handle.text()?)?;
        for i in 0..self.handle.len()? {
            new_handle.insert(i, self.handle.get(i)?)?;
        }
        if let Some(i) = self.handle.selection()? {
            new_handle.set_selection(i)?;
        }
        self.handle = new_handle;
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>>;

    pub fn set_selection(&mut self, i: usize) -> Result<()>;

    pub fn is_editable(&self) -> Result<bool> {
        Ok(self.editable)
    }

    pub fn set_editable(&mut self, v: bool) -> Result<()> {
        if self.editable != v {
            self.recreate(v)?;
            self.editable = v;
        }
        Ok(())
    }

    pub async fn wait_select(&self) {
        self.handle.wait_select().await
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.handle.insert(i, s)?;
        if (!self.is_editable()?) && self.len()? == 1 {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.handle.remove(i, self.editable)
    }

    pub fn get(&self, i: usize) -> Result<String>;

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()>;

    pub fn len(&self) -> Result<usize>;

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()>;
}

winio_handle::impl_as_widget!(ComboBox, handle);
