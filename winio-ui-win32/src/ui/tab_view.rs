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
        WM_NOTIFY, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsRawContainer, AsRawWidget, BorrowedContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::{View, Widget, WindowMessageNotify, ui::with_u16c};

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
    current_view: Cell<Option<usize>>,
    views: Vec<TabViewItem>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Self {
        let mut handle = Widget::new(
            WC_TABCONTROLW,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | TCS_TABS,
            0,
            parent.as_container().as_win32(),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        Self {
            handle,
            current_view: Cell::new(None),
            views: vec![],
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        let mut rect = MaybeUninit::uninit();
        syscall!(
            BOOL,
            GetClientRect(self.handle.as_raw_widget().as_win32(), rect.as_mut_ptr())
        )
        .unwrap();
        self.handle
            .send_message(TCM_ADJUSTRECT, 0, rect.as_mut_ptr() as _);
        let rect = unsafe { rect.assume_init() };
        for item in self.views.iter() {
            unsafe {
                MoveWindow(
                    item.as_raw_container().as_win32(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    1,
                );
            }
        }
    }

    pub fn selection(&self) -> Option<usize> {
        let i = self.handle.send_message(TCM_GETCURSEL, 0, 0);
        if i < 0 { None } else { Some(i as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        if !self.is_empty() && i.is_none() {
            panic!("cannot cancel selection if the tab collection is not empty");
        }
        let i = if let Some(i) = i { i as isize } else { -1 };
        self.handle.send_message(TCM_SETCURSEL, i as _, 0);
    }

    pub async fn start(&self) -> ! {
        loop {
            let WindowMessageNotify {
                hwnd_from, code, ..
            } = self.handle.wait_parent(WM_NOTIFY).await.notify();
            if std::ptr::eq(hwnd_from, self.handle.as_raw_widget().as_win32())
                && (code == TCN_SELCHANGE)
            {
                self.show_current_view();
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
            let sel = self.selection();
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

    pub fn insert(&mut self, i: usize, item: &TabViewItem) {
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
            });
        }
        self.reset_indices();
        if self.len() == 1 {
            self.show_current_view();
        }
    }

    pub fn remove(&mut self, i: usize) {
        let cur = self.selection();
        let need_reselect = cur == Some(i);
        self.handle.send_message(TCM_DELETEITEM, i, 0);
        self.views.remove(i);
        self.reset_indices();
        if need_reselect {
            self.set_selection(Some(0));
        }
    }

    pub fn len(&self) -> usize {
        self.handle.send_message(TCM_GETITEMCOUNT, 0, 0) as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.handle.send_message(TCM_DELETEALLITEMS, 0, 0);
        for item in self.views.drain(..) {
            item.inner.borrow_mut().index = None;
        }
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
    pub fn new(parent: &TabView) -> Self {
        Self {
            inner: Rc::new(RefCell::new(TabViewItemInner {
                view: View::new_hidden(parent.as_raw_widget().as_win32()),
                title: String::new(),
                index: None,
            })),
        }
    }

    pub fn text(&self) -> String {
        self.inner.borrow().title.clone()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
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
                        )
                    });
                }
            }
        }
    }

    pub fn size(&self) -> Size {
        self.inner.borrow().view.size()
    }
}

impl AsRawContainer for TabViewItem {
    fn as_raw_container(&self) -> RawContainer {
        self.inner.borrow().view.as_raw_container()
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}
