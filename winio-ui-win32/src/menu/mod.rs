use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ptr::null,
};

use inherit_methods_macro::inherit_methods;
use widestring::U16CString;
use windows_core::WIN32_ERROR;
use windows_sys::Win32::{
    Foundation::{ERROR_INVALID_PARAMETER, HWND},
    UI::WindowsAndMessaging::{
        CreateMenu, DestroyMenu, GetMenu, GetMenuItemInfoW, HMENU, InsertMenuW, MENUITEMINFOW,
        MF_BYCOMMAND, MF_POPUP, MF_SEPARATOR, MF_UNCHECKED, MFT_RADIOCHECK, MIIM_STRING, SetMenu,
        SetMenuItemInfoW, WM_COMMAND,
    },
};
use winio_handle::{
    AsMenuBar, AsMenuContainer, AsMenuItem, AsWindow, BorrowedMenuBar, BorrowedMenuContainer,
    BorrowedMenuItem,
};
use winio_ui_windows_common::syscall;

use crate::{Error, Result, WindowMessageCommand, get_u16c, with_u16c};

#[derive(Debug)]
pub(crate) struct OwnedMenu(HMENU);

impl OwnedMenu {
    pub fn new() -> Result<Self> {
        let hmenu = unsafe { CreateMenu() };
        if hmenu.is_null() {
            return Err(Error::from_thread());
        }
        Ok(Self(hmenu))
    }
}

impl Drop for OwnedMenu {
    fn drop(&mut self) {
        unsafe { DestroyMenu(self.0) };
    }
}

impl AsMenuBar for OwnedMenu {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        unsafe { BorrowedMenuBar::win32(self.0) }
    }
}

impl AsMenuContainer for OwnedMenu {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        unsafe { BorrowedMenuContainer::win32(self.0) }
    }
}

thread_local! {
    static HMENU_HWND_MAP: RefCell<HashMap<HMENU, HWND>> = RefCell::new(HashMap::new());
    static HMENU_PARENT_MAP: RefCell<HashMap<HMENU, HMENU>> = RefCell::new(HashMap::new());
}

fn parent_window(mut m: HMENU) -> Option<HWND> {
    HMENU_PARENT_MAP.with_borrow(|map| {
        while let Some(parent) = map.get(&m) {
            m = *parent;
        }
    });
    HMENU_HWND_MAP.with_borrow(|map| map.get(&m).copied())
}

#[derive(Debug)]
pub struct MenuBar {
    inner: OwnedMenu,
}

impl MenuBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let parent = parent.as_window().as_win32();
        let old_mean = unsafe { GetMenu(parent) };
        if !old_mean.is_null() {
            return Err(WIN32_ERROR(ERROR_INVALID_PARAMETER as _)
                .to_hresult()
                .into());
        }
        let inner = OwnedMenu::new()?;
        syscall!(BOOL, SetMenu(parent, inner.0))?;
        HMENU_HWND_MAP.with_borrow_mut(|map| map.insert(inner.0, parent));
        Ok(Self { inner })
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        HMENU_HWND_MAP.with_borrow_mut(|map| map.remove(&self.inner.0));
    }
}

winio_handle::impl_as_menu_bar!(MenuBar, inner);

fn menu_id() -> u32 {
    thread_local! {
        static MENU_ID: Cell<u32> = const { Cell::new(0) };
    }
    MENU_ID.with(|id| {
        let old_id = id.get();
        id.set(old_id.saturating_add(1));
        old_id
    })
}

#[derive(Debug)]
struct MenuItemImpl {
    parent: HMENU,
    id: u32,
}

impl MenuItemImpl {
    pub fn new(parent: HMENU, flags: u32) -> Result<Self> {
        let id = menu_id();
        syscall!(
            BOOL,
            InsertMenuW(parent, id, MF_BYCOMMAND | flags, id as _, null())
        )?;
        Ok(Self { parent, id })
    }

    pub fn new_with(parent: HMENU, item: HMENU) -> Result<Self> {
        let id = menu_id();
        syscall!(
            BOOL,
            InsertMenuW(parent, id, MF_BYCOMMAND | MF_POPUP, item as _, null())
        )?;
        Ok(Self { parent, id })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.text_u16()?.to_string_lossy())
    }

    pub fn text_u16(&self) -> Result<U16CString> {
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = size_of::<MENUITEMINFOW>() as _;
        info.fMask = MIIM_STRING;
        syscall!(BOOL, GetMenuItemInfoW(self.parent, self.id, 0, &mut info))?;
        unsafe {
            get_u16c(info.cch as _, |buf| {
                info.dwTypeData = buf.as_mut_ptr().cast();
                syscall!(BOOL, GetMenuItemInfoW(self.parent, self.id, 0, &mut info))?;
                Ok(info.cch as _)
            })
        }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        with_u16c(s.as_ref(), |s| {
            let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
            info.cbSize = size_of::<MENUITEMINFOW>() as _;
            info.fMask = MIIM_STRING;
            info.dwTypeData = s.as_ptr() as _;
            info.cch = s.len() as _;
            syscall!(BOOL, SetMenuItemInfoW(self.parent, self.id, 0, &info))?;
            Ok(())
        })
    }

    pub async fn wait_click(&self) {
        let parent_window = parent_window(self.parent);
        if let Some(hwnd) = parent_window {
            loop {
                let WindowMessageCommand { id, .. } =
                    unsafe { crate::wait(hwnd, WM_COMMAND).await.command() };
                if id == self.id {
                    break;
                }
            }
        } else {
            std::future::pending::<()>().await;
        }
    }
}

impl AsMenuItem for MenuItemImpl {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        unsafe { BorrowedMenuItem::win32(self.parent, self.id) }
    }
}

#[derive(Debug)]
pub struct MenuBarItem {
    inner: OwnedMenu,
    item: MenuItemImpl,
}

#[inherit_methods(from = "self.item")]
impl MenuBarItem {
    pub fn new(parent: impl AsMenuBar) -> Result<Self> {
        let parent = parent.as_menu_bar().as_win32();
        let inner = OwnedMenu::new()?;
        let item = MenuItemImpl::new_with(parent, inner.0)?;
        HMENU_PARENT_MAP.with_borrow_mut(|map| map.insert(inner.0, parent));
        Ok(Self { inner, item })
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl Drop for MenuBarItem {
    fn drop(&mut self) {
        HMENU_PARENT_MAP.with_borrow_mut(|map| map.remove(&self.inner.0));
    }
}

winio_handle::impl_as_menu_container!(MenuBarItem, inner);

#[derive(Debug)]
pub struct MenuItem {
    item: MenuItemImpl,
}

#[inherit_methods(from = "self.item")]
impl MenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_win32();
        let item = MenuItemImpl::new(parent, 0)?;
        Ok(Self { item })
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

winio_handle::impl_as_menu_item!(MenuItem, item);

#[derive(Debug)]
pub struct PopupMenuItem {
    inner: OwnedMenu,
    item: MenuItemImpl,
}

#[inherit_methods(from = "self.item")]
impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_win32();
        let inner = OwnedMenu::new()?;
        let item = MenuItemImpl::new_with(parent, inner.0)?;
        HMENU_PARENT_MAP.with_borrow_mut(|map| map.insert(inner.0, parent));
        Ok(Self { inner, item })
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl Drop for PopupMenuItem {
    fn drop(&mut self) {
        HMENU_PARENT_MAP.with_borrow_mut(|map| map.remove(&self.inner.0));
    }
}

winio_handle::impl_as_menu_container!(PopupMenuItem, inner);
winio_handle::impl_as_menu_item!(PopupMenuItem, item);

#[derive(Debug)]
pub struct CheckMenuItem {
    item: MenuItemImpl,
}

#[inherit_methods(from = "self.item")]
impl CheckMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_win32();
        let item = MenuItemImpl::new(parent, MF_UNCHECKED)?;
        Ok(Self { item })
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

winio_handle::impl_as_menu_item!(CheckMenuItem, item);

#[derive(Debug)]
pub struct RadioMenuItem {
    item: MenuItemImpl,
}

#[inherit_methods(from = "self.item")]
impl RadioMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_win32();
        let item = MenuItemImpl::new(parent, MF_UNCHECKED)?;
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = size_of::<MENUITEMINFOW>() as _;
        syscall!(BOOL, GetMenuItemInfoW(parent, item.id, 0, &mut info))?;
        info.fType |= MFT_RADIOCHECK;
        syscall!(BOOL, SetMenuItemInfoW(parent, item.id, 0, &info))?;
        Ok(Self { item })
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, item);

#[derive(Debug)]
pub struct SeparatorMenuItem {
    item: MenuItemImpl,
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_win32();
        let item = MenuItemImpl::new(parent, MF_SEPARATOR)?;
        Ok(Self { item })
    }
}

winio_handle::impl_as_menu_item!(SeparatorMenuItem, item);
