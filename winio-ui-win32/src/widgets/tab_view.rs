use std::{
    cell::RefCell,
    mem::MaybeUninit,
    rc::{Rc, Weak},
};

use inherit_methods_macro::inherit_methods;
use windows_core::WIN32_ERROR;
use windows_sys::{
    Win32::{
        Foundation::ERROR_ALREADY_EXISTS,
        UI::{
            Controls::{
                TCIF_TEXT, TCITEMW, TCM_ADJUSTRECT, TCM_DELETEALLITEMS, TCM_DELETEITEM,
                TCM_GETCURSEL, TCM_INSERTITEMW, TCM_SETCURSEL, TCM_SETITEMW, TCN_SELCHANGE,
                TCS_TABS, WC_TABCONTROLW,
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
use winio_handle::{AsContainer, AsWidget, BorrowedContainer, BorrowedWidget};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::{children_refresh_dark_mode, syscall};

use crate::{Result, View, Widget, WindowMessageNotify, widgets::with_u16c};

#[derive(Debug)]
pub struct TabView {
    data: Rc<TabViewData>,
}

#[derive(Debug)]
struct TabViewData {
    handle: Widget,
    views: RefCell<Vec<TabViewItem>>,
}

#[inherit_methods(from = "self.data.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
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

        let data = Rc::new(TabViewData {
            handle,
            views: RefCell::new(vec![]),
        });
        Ok(Self { data })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        let data = &self.data;
        data.handle.set_size(v)?;
        let mut rect = MaybeUninit::uninit();
        syscall!(
            BOOL,
            GetClientRect(data.handle.as_widget().as_win32(), rect.as_mut_ptr())
        )?;
        data.handle
            .send_message(TCM_ADJUSTRECT, 0, rect.as_mut_ptr() as _);
        let rect = unsafe { rect.assume_init() };
        for item in data.views.borrow().iter() {
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

    pub fn selection(&self) -> Result<Option<usize>> {
        Ok(self.data.selection())
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.data.handle.send_message(TCM_SETCURSEL, i as _, 0);
        Ok(())
    }

    pub async fn wait_select(&self) {
        loop {
            let WindowMessageNotify {
                hwnd_from, code, ..
            } = self.data.handle.wait_parent(WM_NOTIFY).await.notify();
            if std::ptr::eq(hwnd_from, self.data.handle.as_widget().as_win32())
                && (code == TCN_SELCHANGE)
            {
                self.data.show_current_view();
                return;
            }
        }
    }

    fn append(&self, item: &TabViewItem) -> Result<()> {
        self.data.append(item)
    }
}

impl TabViewData {
    fn selection(&self) -> Option<usize> {
        let i = self.handle.send_message(TCM_GETCURSEL, 0, 0);
        if i < 0 { None } else { Some(i as _) }
    }

    fn show_current_view(&self) {
        let sel = self.selection();
        let views = self.views.borrow();
        for (i, item) in views.iter().enumerate() {
            let hwnd = item.inner.borrow().view.as_widget().as_win32();
            unsafe {
                ShowWindow(hwnd, if Some(i) == sel { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    fn reset_indices(&self) {
        for (i, item) in self.views.borrow().iter().enumerate() {
            item.inner.borrow_mut().index = Some(i);
        }
    }

    fn append(&self, item: &TabViewItem) -> Result<()> {
        if item.inner.borrow().index.is_some() {
            return Err(WIN32_ERROR(ERROR_ALREADY_EXISTS).to_hresult().into());
        }
        let item_hwnd = item.as_container().as_win32();
        let tab_hwnd = self.handle.as_widget().as_win32();
        let previous_parent = unsafe { GetParent(item_hwnd) };
        if previous_parent == tab_hwnd {
            return Err(WIN32_ERROR(ERROR_ALREADY_EXISTS).to_hresult().into());
        }
        unsafe { SetParent(item_hwnd, tab_hwnd) };

        let i = self.views.borrow().len();
        self.views.borrow_mut().push(item.clone());
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
        if i == 0 {
            self.show_current_view();
        }
        Ok(())
    }

    fn remove(&self, item: &TabViewItem) {
        let index = {
            let views = self.views.borrow();
            let Some(index) = views
                .iter()
                .position(|view| Rc::ptr_eq(&view.inner, &item.inner))
            else {
                return;
            };
            index
        };
        {
            let mut inner = item.inner.borrow_mut();
            inner.index = None;
            unsafe { SetParent(inner.view.as_widget().as_win32(), HWND_MESSAGE) };
        }
        self.handle.send_message(TCM_DELETEITEM, index, 0);
        let removed = {
            let mut views = self.views.borrow_mut();
            views.remove(index)
        };
        drop(removed);
        self.reset_indices();
        if self.selection().is_none() && !self.views.borrow().is_empty() {
            self.handle.send_message(TCM_SETCURSEL, 0, 0);
        }
        self.show_current_view();
    }
}

impl AsWidget for TabView {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        self.data.handle.as_widget()
    }
}

#[derive(Debug)]
struct TabViewItemInner {
    view: View,
    title: String,
    index: Option<usize>,
    parent: Weak<TabViewData>,
}

#[derive(Debug, Clone)]
pub struct TabViewItem {
    inner: Rc<RefCell<TabViewItemInner>>,
}

impl TabViewItem {
    pub fn new(parent: &TabView) -> Result<Self> {
        let item = Self {
            inner: Rc::new(RefCell::new(TabViewItemInner {
                view: View::new_hidden(HWND_MESSAGE)?,
                title: String::new(),
                index: None,
                parent: Rc::downgrade(&parent.data),
            })),
        };
        parent.append(&item)?;
        Ok(item)
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

impl Drop for TabViewItem {
    fn drop(&mut self) {
        let parent = self.inner.borrow().parent.upgrade();
        if let Some(parent) = parent {
            parent.remove(self);
        }
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        // SAFETY: view is not replaced after creation
        unsafe { BorrowedContainer::win32(self.inner.borrow().view.as_container().as_win32()) }
    }
}
