use std::{mem::zeroed, ptr::null, rc::Rc};

use windows_sys::Win32::{
    Foundation::{LPARAM, POINT, RECT, WPARAM},
    Graphics::Gdi::ClientToScreen,
    UI::{
        Controls::{TB_GETRECT, TBN_DROPDOWN},
        WindowsAndMessaging::{
            SetForegroundWindow, TPM_LEFTALIGN, TPM_TOPALIGN, TrackPopupMenuEx, WM_COMMAND,
            WM_NOTIFY,
        },
    },
};
use winio_handle::{AsToolBarItem, AsWindow, BorrowedToolBarItem};

use super::{ItemKind, ToolBarData, bar::ToolBar};
use crate::{Image, Result, menu::OwnedMenu};

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    data: Rc<ToolBarData>,
    id: u32,
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        unsafe { BorrowedToolBarItem::win32(self.data.handle.as_window().as_win32(), self.id) }
    }
}

impl ToolBarItem {
    pub(crate) fn new(parent: &ToolBar, kind: ItemKind) -> Result<Self> {
        Ok(Self {
            data: parent.data().clone(),
            id: parent.data().add_item(kind)?,
        })
    }

    pub(crate) fn is_enabled(&self) -> Result<bool> {
        self.data.is_enabled(self.id)
    }

    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.data.set_enabled(self.id, enabled)
    }

    pub(crate) fn text(&self) -> Result<String> {
        self.data.text(self.id)
    }

    pub(crate) fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
        self.data.set_text(self.id, text.as_ref())
    }

    pub(crate) fn is_checked(&self) -> Result<bool> {
        self.data.is_checked(self.id)
    }

    pub(crate) fn set_checked(&self, checked: bool) -> Result<()> {
        self.data.set_checked(self.id, checked)
    }

    pub(crate) fn set_icon(&self, icon: Option<&Image>) -> Result<()> {
        self.data.set_icon(self.id, icon)
    }

    pub(crate) async fn wait_click(&self) {
        loop {
            let command = self.data.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(command.handle, self.data.handle.as_window().as_win32())
                && command.id == self.id
            {
                break;
            }
        }
    }

    pub(crate) async fn wait_drop_down(&self) {
        loop {
            let notify = self.data.handle.wait_parent(WM_NOTIFY).await.notify();
            if std::ptr::eq(notify.hwnd_from, self.data.handle.as_window().as_win32())
                && notify.code == TBN_DROPDOWN
                && notify.item == self.id as i32
            {
                break;
            }
        }
    }

    pub(crate) fn show_drop_down(&self, menu: &OwnedMenu) {
        let mut rect: RECT = unsafe { zeroed() };
        let res = self.data.handle.send_message(
            TB_GETRECT,
            self.id as WPARAM,
            &mut rect as *mut RECT as LPARAM,
        );
        if res == 0 {
            return;
        }
        let mut point = POINT {
            x: rect.left,
            y: rect.bottom,
        };
        if unsafe { ClientToScreen(self.data.handle.as_window().as_win32(), &mut point) } == 0 {
            return;
        }
        unsafe {
            let parent = self.data.handle.parent();
            SetForegroundWindow(parent);
            TrackPopupMenuEx(
                menu.0,
                TPM_LEFTALIGN | TPM_TOPALIGN,
                point.x,
                point.y,
                parent,
                null(),
            );
        }
    }
}
