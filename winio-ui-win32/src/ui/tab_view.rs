use std::{
    cell::{Cell, RefCell},
    mem::MaybeUninit,
    rc::Rc,
};

use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{
        TCIF_TEXT, TCITEMW, TCM_ADJUSTRECT, TCM_DELETEALLITEMS, TCM_DELETEITEM, TCM_GETCURSEL,
        TCM_GETITEMCOUNT, TCM_INSERTITEMW, TCM_SETCURSEL, TCM_SETITEMW, TCN_SELCHANGE, TCS_TABS,
        WC_TABCONTROLW,
    },
    WindowsAndMessaging::{
        GetClientRect, GetParent, MoveWindow, SW_HIDE, SW_SHOW, SendMessageW, ShowWindow,
        WM_NOTIFY, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT, WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsRawContainer, AsRawWidget, RawContainer};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::children_refresh_dark_mode;

use crate::{Result, View, Widget, WindowMessageNotify, ui::with_u16c};

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
    current_view: Cell<Option<usize>>,
    views: Vec<TabViewItem>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_TABCONTROLW,
            WS_TABSTOP | WS_CLIPCHILDREN | WS_VISIBLE | WS_CHILD | TCS_TABS,
            WS_EX_CONTROLPARENT,
            parent.as_container().as_win32(),
        )?;
        Ok(Self {
            handle,
            current_view: Cell::new(None),
            views: vec![],
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        let mut rect = MaybeUninit::uninit();
        syscall!(
            BOOL,
            GetClientRect(self.handle.as_raw_widget().as_win32(), rect.as_mut_ptr())
        )?;
        self.handle
            .send_message(TCM_ADJUSTRECT, 0, rect.as_mut_ptr() as _);
        let rect = unsafe { rect.assume_init() };
        for item in self.views.iter() {
            syscall!(
                BOOL,
                MoveWindow(
                    item.as_raw_container().as_win32(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    1,
                )
            )?;
        }
        Ok(())
    }

    fn selection_impl(&self) -> Option<usize> {
        let i = self.handle.send_message(TCM_GETCURSEL, 0, 0);
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        Ok(self.selection_impl())
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.handle.send_message(TCM_SETCURSEL, i as _, 0);
        Ok(())
    }

    pub async fn wait_select(&self) {
        loop {
            let WindowMessageNotify {
                hwnd_from, code, ..
            } = self.handle.wait_parent(WM_NOTIFY).await.notify();
            if std::ptr::eq(hwnd_from, self.handle.as_raw_widget().as_win32())
                && (code == TCN_SELCHANGE)
            {
                self.show_current_view();
                return;
            }
        }
    }

    fn show_current_view(&self) {
        unsafe {
            let current_view = self.current_view.get();
            if let Some(index) = current_view {
                if let Some(view) = self.views.get(index) {
                    ShowWindow(view.inner.borrow().view.as_raw_widget().as_win32(), SW_HIDE);
                }
            }
            let sel = self.selection_impl();
            self.current_view.set(sel);
            if let Some(sel) = sel {
                if let Some(view) = self.views.get(sel) {
                    ShowWindow(view.inner.borrow().view.as_raw_widget().as_win32(), SW_SHOW);
                }
            }
        }
    }

    fn reset_indices(&mut self) {
        for (i, item) in self.views.iter().enumerate() {
            item.inner.borrow_mut().index = Some(i);
        }
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        self.views.insert(i, item.clone());
        {
            let mut inner = item.inner.borrow_mut();
            inner.index = Some(i);
            with_u16c(&inner.title, |s| {
                let mut item = TCITEMW {
                    mask: TCIF_TEXT,
                    dwState: 0,
                    dwStateMask: 0,
                    pszText: s.as_ptr().cast_mut(),
                    cchTextMax: 0,
                    iImage: 0,
                    lParam: 0,
                };
                self.handle
                    .send_message(TCM_INSERTITEMW, i, std::ptr::addr_of_mut!(item) as _);
                Ok(())
            })?;
            // The updown control is created lazily.
            unsafe {
                children_refresh_dark_mode(self.handle.as_raw_widget().as_win32(), 0);
            }
        }
        self.reset_indices();
        if self.len()? == 1 {
            self.show_current_view();
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        let cur = self.selection()?;
        let need_reselect = cur == Some(i);
        self.handle.send_message(TCM_DELETEITEM, i, 0);
        self.views.remove(i);
        self.reset_indices();
        if need_reselect {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.handle.send_message(TCM_GETITEMCOUNT, 0, 0) as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.handle.send_message(TCM_DELETEALLITEMS, 0, 0);
        for item in self.views.drain(..) {
            item.inner.borrow_mut().index = None;
        }
        Ok(())
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
struct TabViewItemInner {
    view: View,
    title: String,
    index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TabViewItem {
    inner: Rc<RefCell<TabViewItemInner>>,
}

impl TabViewItem {
    pub fn new(parent: &TabView) -> Result<Self> {
        Ok(Self {
            inner: Rc::new(RefCell::new(TabViewItemInner {
                view: View::new_hidden(parent.as_raw_widget().as_win32())?,
                title: String::new(),
                index: None,
            })),
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.inner.borrow().title.clone())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        inner.title = s.as_ref().to_string();
        unsafe {
            let parent = GetParent(inner.view.as_raw_widget().as_win32());
            if let Some(index) = inner.index {
                if !parent.is_null() {
                    with_u16c(&inner.title, |s| {
                        let mut item = TCITEMW {
                            mask: TCIF_TEXT,
                            dwState: 0,
                            dwStateMask: 0,
                            pszText: s.as_ptr().cast_mut(),
                            cchTextMax: 0,
                            iImage: 0,
                            lParam: 0,
                        };
                        SendMessageW(
                            parent,
                            TCM_SETITEMW,
                            index,
                            std::ptr::addr_of_mut!(item) as _,
                        );
                        Ok(())
                    })?;
                }
            }
        }
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        self.inner.borrow().view.size()
    }
}

impl AsRawContainer for TabViewItem {
    fn as_raw_container(&self) -> RawContainer {
        self.inner.borrow().view.as_raw_container()
    }
}

winio_handle::impl_as_container!(TabViewItem);
