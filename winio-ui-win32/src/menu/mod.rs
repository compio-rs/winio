use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ptr::null_mut,
};

use inherit_methods_macro::inherit_methods;
use widestring::U16CString;
use windows_core::WIN32_ERROR;
use windows_sys::Win32::{
    Foundation::{ERROR_INVALID_PARAMETER, HWND},
    Graphics::Gdi::HBITMAP,
    UI::{
        HiDpi::GetSystemMetricsForDpi,
        WindowsAndMessaging::{
            CheckMenuItem, CreateMenu, DeleteMenu, DestroyMenu, DrawMenuBar, EnableMenuItem,
            GetMenu, GetMenuItemInfoW, GetMenuState, HMENU, InsertMenuItemW, MENUITEMINFOW,
            MF_BYCOMMAND, MF_CHECKED, MF_DISABLED, MF_ENABLED, MF_GRAYED, MF_SEPARATOR,
            MF_UNCHECKED, MFT_RADIOCHECK, MIIM_BITMAP, MIIM_FTYPE, MIIM_ID, MIIM_STRING,
            MIIM_SUBMENU, SM_CXMENUCHECK, SM_CYMENUCHECK, SetMenu, SetMenuItemInfoW, WM_COMMAND,
        },
    },
};
use winio_handle::{
    AsMenuBar, AsMenuContainer, AsMenuItem, AsWindow, BorrowedMenuBar, BorrowedMenuContainer,
    BorrowedMenuItem,
};
use winio_ui_windows_common::syscall;

use crate::{
    Error, Result, WindowMessageCommand, get_u16c,
    platform::{
        dpi::get_dpi_for_window,
        image::{Image, WinBitmap},
    },
    with_u16c,
};

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

struct MenuIcon {
    _bitmap: WinBitmap,
    image: Image,
    size: (i32, i32),
}

thread_local! {
    static HMENU_ICONS: RefCell<HashMap<(HMENU, u32), MenuIcon>> = RefCell::new(HashMap::new());
}

fn menu_icon_size(parent: HMENU) -> (i32, i32) {
    let dpi = get_dpi_for_window(parent_window(parent).unwrap_or_default());
    unsafe {
        (
            GetSystemMetricsForDpi(SM_CXMENUCHECK, dpi),
            GetSystemMetricsForDpi(SM_CYMENUCHECK, dpi),
        )
    }
}

fn set_item_bitmap(parent: HMENU, id: u32, bitmap: HBITMAP) -> Result<()> {
    let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MENUITEMINFOW>() as _;
    info.fMask = MIIM_BITMAP;
    info.hbmpItem = bitmap;
    syscall!(BOOL, SetMenuItemInfoW(parent, id, 0, &info))?;
    Ok(())
}

pub(crate) fn refresh_menu_icons(hwnd: HWND) -> Result<()> {
    let keys = HMENU_ICONS.with_borrow(|icons| {
        icons
            .keys()
            .copied()
            .filter(|&(parent, _)| parent_window(parent) == Some(hwnd))
            .collect::<Vec<_>>()
    });

    for key in keys {
        let Some((image, old_size)) = HMENU_ICONS
            .with_borrow(|icons| icons.get(&key).map(|icon| (icon.image.clone(), icon.size)))
        else {
            continue;
        };

        let size = menu_icon_size(key.0);
        if size == old_size {
            continue;
        }

        let bitmap = image.to_hbitmap(size)?;
        set_item_bitmap(key.0, key.1, bitmap.0)?;
        HMENU_ICONS.with_borrow_mut(|icons| {
            icons.insert(
                key,
                MenuIcon {
                    _bitmap: bitmap,
                    image,
                    size,
                },
            )
        });
    }

    syscall!(BOOL, DrawMenuBar(hwnd))?;
    Ok(())
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
        syscall!(BOOL, DrawMenuBar(parent))?;
        HMENU_HWND_MAP.with_borrow_mut(|map| map.insert(inner.0, parent));
        Ok(Self { inner })
    }

    pub fn is_visible(&self) -> Result<bool> {
        let parent_window = parent_window(self.inner.0);
        if let Some(hwnd) = parent_window {
            let old_mean = unsafe { GetMenu(hwnd) };
            Ok(old_mean == self.inner.0)
        } else {
            Ok(false)
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        let parent_window = parent_window(self.inner.0);
        if let Some(hwnd) = parent_window {
            if visible {
                syscall!(BOOL, SetMenu(hwnd, self.inner.0))?;
            } else {
                syscall!(BOOL, SetMenu(hwnd, std::ptr::null_mut()))?;
            }
            syscall!(BOOL, DrawMenuBar(hwnd))?;
        }
        Ok(())
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
        let mut text = [0u16];
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = size_of::<MENUITEMINFOW>() as _;
        info.fMask = MIIM_ID | MIIM_FTYPE | MIIM_STRING;
        info.fType = flags;
        info.wID = id;
        info.dwTypeData = text.as_mut_ptr();
        info.cch = 0;
        syscall!(BOOL, InsertMenuItemW(parent, id, 0, &info))?;
        if let Some(w) = parent_window(parent) {
            syscall!(BOOL, DrawMenuBar(w))?;
        }
        Ok(Self { parent, id })
    }

    pub fn new_with(parent: HMENU, item: HMENU) -> Result<Self> {
        let id = menu_id();
        let mut text = [0u16];
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = size_of::<MENUITEMINFOW>() as _;
        info.fMask = MIIM_ID | MIIM_SUBMENU | MIIM_STRING;
        info.wID = id;
        info.hSubMenu = item;
        info.dwTypeData = text.as_mut_ptr();
        info.cch = 0;
        syscall!(BOOL, InsertMenuItemW(parent, id, 0, &info))?;
        if let Some(w) = parent_window(parent) {
            syscall!(BOOL, DrawMenuBar(w))?;
        }
        Ok(Self { parent, id })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        let state = unsafe { GetMenuState(self.parent, self.id, MF_BYCOMMAND) };
        Ok((state & (MF_DISABLED | MF_GRAYED)) == 0)
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        unsafe {
            EnableMenuItem(
                self.parent,
                self.id,
                MF_BYCOMMAND
                    | if enabled {
                        MF_ENABLED
                    } else {
                        MF_DISABLED | MF_GRAYED
                    },
            )
        };
        Ok(())
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
                info.cch = buf.len() as _;
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
        })?;
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        let key = (self.parent, self.id);

        let icon = match icon {
            Some(image) => {
                let size = menu_icon_size(self.parent);
                let bitmap = image.to_hbitmap(size)?;
                set_item_bitmap(self.parent, self.id, bitmap.0)?;
                Some(MenuIcon {
                    _bitmap: bitmap,
                    image: image.clone(),
                    size,
                })
            }
            None => {
                set_item_bitmap(self.parent, self.id, null_mut())?;
                None
            }
        };

        HMENU_ICONS.with_borrow_mut(|icons| match icon {
            Some(icon) => icons.insert(key, icon),
            None => icons.remove(&key),
        });

        if let Some(hwnd) = parent_window(self.parent) {
            syscall!(BOOL, DrawMenuBar(hwnd))?;
        }
        Ok(())
    }

    pub fn is_checked_impl(&self) -> bool {
        let state = unsafe { GetMenuState(self.parent, self.id, MF_BYCOMMAND) };
        (state & MF_CHECKED) != 0
    }

    pub fn is_checked(&self) -> Result<bool> {
        Ok(self.is_checked_impl())
    }

    pub fn set_checked_impl(&self, checked: bool) {
        unsafe {
            CheckMenuItem(
                self.parent,
                self.id,
                MF_BYCOMMAND | if checked { MF_CHECKED } else { MF_UNCHECKED },
            )
        };
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        self.set_checked_impl(checked);
        Ok(())
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

impl Drop for MenuItemImpl {
    fn drop(&mut self) {
        let _ = syscall!(BOOL, DeleteMenu(self.parent, self.id, MF_BYCOMMAND));
        HMENU_ICONS.with_borrow_mut(|icons| icons.remove(&(self.parent, self.id)));
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

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

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

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

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

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;
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
        let item = MenuItemImpl::new(parent, 0)?;
        Ok(Self { item })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
        self.item.set_checked_impl(!self.item.is_checked_impl());
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
        let item = MenuItemImpl::new(parent, MFT_RADIOCHECK)?;
        Ok(Self { item })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
        self.item.set_checked_impl(true);
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
