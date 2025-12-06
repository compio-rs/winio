use std::{
    cell::{Cell, RefCell},
    mem::MaybeUninit,
    rc::Rc,
};

use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows::core::HRESULT;
use windows_sys::{
    Win32::{
        Foundation::{ERROR_ALREADY_EXISTS, ERROR_HWNDS_HAVE_DIFF_PARENT},
        UI::{
            Controls::{
                TCIF_TEXT, TCITEMW, TCM_ADJUSTRECT, TCM_DELETEALLITEMS, TCM_DELETEITEM,
                TCM_GETCURSEL, TCM_GETITEMCOUNT, TCM_INSERTITEMW, TCM_SETCURSEL, TCM_SETITEMW,
                TCN_SELCHANGE, TCS_TABS, WC_TABCONTROLW,
            },
            WindowsAndMessaging::{
                GetClientRect, GetParent, HWND_MESSAGE, MoveWindow, SW_HIDE, SW_SHOW, SendMessageW,
                SetParent, ShowWindow, WM_NOTIFY, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT,
                WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    w,
};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::children_refresh_dark_mode;

use crate::{Error, Result, View, Widget, WindowMessageNotify, ui::with_u16c};

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
    current_view: Cell<Option<usize>>,
    views: Vec<TabViewItem>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut handle = Widget::new(
            WC_TABCONTROLW,
            WS_TABSTOP | WS_CLIPCHILDREN | WS_VISIBLE | WS_CHILD | TCS_TABS,
            WS_EX_CONTROLPARENT,
            parent.as_container().as_win32(),
        )?;

        // The inner updown control is created when
        // * the tab control is small enough
        // * there are at least two tabs with long enough text
        //
        // So we create two dummy tabs with long enough text to trigger it.
        handle.set_size(handle.size_d2l((1, 1)))?;
        let text = w!("DummyTabLongEnoughToCreateUpdown");
        let mut item = TCITEMW {
            mask: TCIF_TEXT,
            dwState: 0,
            dwStateMask: 0,
            pszText: text.cast_mut(),
            cchTextMax: 0,
            iImage: 0,
            lParam: 0,
        };
        handle.send_message(TCM_INSERTITEMW, 0, std::ptr::addr_of_mut!(item) as _);
        handle.send_message(TCM_INSERTITEMW, 1, std::ptr::addr_of_mut!(item) as _);
        unsafe { children_refresh_dark_mode(handle.as_widget().as_win32(), 0) };
        handle.send_message(TCM_DELETEALLITEMS, 0, 0);

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
            GetClientRect(self.handle.as_widget().as_win32(), rect.as_mut_ptr())
        )?;
        self.handle
            .send_message(TCM_ADJUSTRECT, 0, rect.as_mut_ptr() as _);
        let rect = unsafe { rect.assume_init() };
        for item in self.views.iter() {
            syscall!(
                BOOL,
                MoveWindow(
                    item.as_container().as_win32(),
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
            if std::ptr::eq(hwnd_from, self.handle.as_widget().as_win32())
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
            if let Some(index) = current_view
                && let Some(view) = self.views.get(index)
            {
                ShowWindow(view.inner.borrow().view.as_widget().as_win32(), SW_HIDE);
            }
            let sel = self.selection_impl();
            self.current_view.set(sel);
            if let Some(sel) = sel
                && let Some(view) = self.views.get(sel)
            {
                ShowWindow(view.inner.borrow().view.as_widget().as_win32(), SW_SHOW);
            }
        }
    }

    fn reset_indices(&mut self) {
        for (i, item) in self.views.iter().enumerate() {
            item.inner.borrow_mut().index = Some(i);
        }
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        if item.inner.borrow().index.is_some() {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_HWNDS_HAVE_DIFF_PARENT,
            )));
        }
        let item_hwnd = item.as_container().as_win32();
        let previous_parent = unsafe { GetParent(item_hwnd) };
        let new_parent = self.as_widget().as_win32();
        if previous_parent == new_parent {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_ALREADY_EXISTS,
            )));
        }
        unsafe { SetParent(item_hwnd, new_parent) };

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
        let item = self.views.remove(i);
        let mut inner = item.inner.borrow_mut();
        unsafe { SetParent(inner.view.as_widget().as_win32(), HWND_MESSAGE) };
        inner.index = None;
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
            let mut inner = item.inner.borrow_mut();
            unsafe { SetParent(inner.view.as_widget().as_win32(), HWND_MESSAGE) };
            inner.index = None;
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
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Rc::new(RefCell::new(TabViewItemInner {
                view: View::new_hidden(HWND_MESSAGE)?,
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
            let parent = GetParent(inner.view.as_widget().as_win32());
            if let Some(index) = inner.index
                && !parent.is_null()
            {
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
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        self.inner.borrow().view.size()
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        // SAFETY: view is not replaced after creation
        unsafe { BorrowedContainer::win32(self.inner.borrow().view.as_container().as_win32()) }
    }
}
