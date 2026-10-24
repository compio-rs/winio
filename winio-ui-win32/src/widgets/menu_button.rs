use std::{mem::zeroed, ptr::null};

use futures_util::FutureExt;
use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::{
    Foundation::RECT,
    UI::{
        Controls::BCN_DROPDOWN,
        WindowsAndMessaging::{
            GetParent, GetWindowRect, SetForegroundWindow, TPM_LEFTALIGN, TPM_TOPALIGN,
            TrackPopupMenuEx, WM_NOTIFY,
        },
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    Image, Result,
    menu::{OwnedMenu, set_menu_owner},
    runtime::WindowMessageNotify,
    widgets::Button,
};

#[derive(Debug)]
pub struct MenuButton {
    button: Button,
    menu: OwnedMenu,
}

#[inherit_methods(from = "self.button")]
impl MenuButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent_hwnd = parent.as_container().as_win32();
        let button = Button::new_split(parent)?;
        let menu = OwnedMenu::new_popup()?;
        set_menu_owner(&menu, parent_hwnd);
        Ok(Self { button, menu })
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

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    fn show_menu(&self) {
        let hwnd = self.button.as_widget().as_win32();
        let parent = unsafe { GetParent(hwnd) };
        let mut rect: RECT = unsafe { zeroed() };
        if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
            return;
        }
        unsafe {
            SetForegroundWindow(parent);
            TrackPopupMenuEx(
                self.menu.0,
                TPM_LEFTALIGN | TPM_TOPALIGN,
                rect.left,
                rect.bottom,
                parent,
                null(),
            );
        }
    }

    pub async fn start(&self) -> ! {
        loop {
            let show = futures_util::select! {
                _ = self.button.wait_click().fuse() => true,
                msg = self.button.wait_parent(WM_NOTIFY).fuse() => {
                    let WindowMessageNotify { hwnd_from, code, .. } = msg.notify();
                    let hwnd = self.button.as_widget().as_win32();
                    std::ptr::eq(hwnd_from, hwnd) && code == BCN_DROPDOWN
                },
            };
            if show {
                self.show_menu();
            }
        }
    }
}

winio_handle::impl_as_widget!(MenuButton, button);
winio_handle::impl_as_menu_container!(MenuButton, menu);
