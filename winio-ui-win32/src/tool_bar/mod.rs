use std::{
    cell::{Cell, RefCell},
    mem::zeroed,
    ptr::null_mut,
    rc::Rc,
};

use widestring::U16CString;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    UI::{
        Controls::{
            BTNS_AUTOSIZE, BTNS_CHECK, BTNS_DROPDOWN, BTNS_SEP, BTNS_SHOWTEXT, BTNS_WHOLEDROPDOWN,
            HIMAGELIST, I_IMAGENONE, ILC_COLOR32, ImageList_Add, ImageList_Create,
            ImageList_Destroy, TB_ADDBUTTONS, TB_AUTOSIZE, TB_BUTTONSTRUCTSIZE, TB_DELETEBUTTON,
            TB_GETBUTTONTEXTW, TB_GETSTATE, TB_SETBITMAPSIZE, TB_SETBUTTONINFOW,
            TB_SETEXTENDEDSTYLE, TB_SETIMAGELIST, TB_SETSTATE, TBBUTTON, TBBUTTONINFOW, TBIF_IMAGE,
            TBIF_SIZE, TBIF_TEXT, TBSTATE_CHECKED, TBSTATE_ENABLED, TBSTYLE_EX_MIXEDBUTTONS,
        },
        HiDpi::GetSystemMetricsForDpi,
        WindowsAndMessaging::{GetWindowRect, IsChild, SM_CXSMICON, SM_CYSMICON},
    },
};
use winio_handle::AsWindow;

use crate::{Error, Image, Result, Widget, get_u16c, with_u16c};

mod bar;
pub use bar::*;

mod button;
pub use button::*;

mod check_box;
pub use check_box::*;

mod drop_down;
pub use drop_down::*;

mod item;
pub(crate) use item::ToolBarItem;

mod radio_button;
pub use radio_button::*;

mod separator;
pub use separator::*;

const TOOLBAR_ID_BASE: u32 = 0x8000;
const SEPARATOR_WIDTH: i32 = 8;

thread_local! {
    static TOOLBARS: RefCell<Vec<Rc<ToolBarData>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemKind {
    Button,
    CheckBox,
    RadioButton,
    DropDownButton,
    Separator,
}

impl ItemKind {
    fn style(self) -> u8 {
        match self {
            Self::Button => (BTNS_AUTOSIZE | BTNS_SHOWTEXT) as u8,
            Self::CheckBox | Self::RadioButton => {
                (BTNS_AUTOSIZE | BTNS_SHOWTEXT | BTNS_CHECK) as u8
            }
            Self::DropDownButton => {
                (BTNS_AUTOSIZE | BTNS_SHOWTEXT | BTNS_DROPDOWN | BTNS_WHOLEDROPDOWN) as u8
            }
            Self::Separator => BTNS_SEP as u8,
        }
    }
}

#[derive(Debug)]
struct WinImageList(HIMAGELIST);

impl Drop for WinImageList {
    fn drop(&mut self) {
        unsafe { ImageList_Destroy(self.0) };
    }
}

#[derive(Debug)]
struct ItemState {
    key: usize,
    id: u32,
    icon: Option<Image>,
    has_text: bool,
}

#[derive(Debug)]
pub(crate) struct ToolBarData {
    handle: Widget,
    items: RefCell<Vec<ItemState>>,
    image_list: RefCell<Option<WinImageList>>,
    next_id: Cell<u32>,
    next_key: Cell<usize>,
}

fn small_icon_size(dpi: u32) -> (i32, i32) {
    unsafe {
        (
            GetSystemMetricsForDpi(SM_CXSMICON, dpi),
            GetSystemMetricsForDpi(SM_CYSMICON, dpi),
        )
    }
}

fn make_long(x: i32, y: i32) -> isize {
    ((x & 0xffff) | (y << 16)) as isize
}

impl ToolBarData {
    fn new(handle: Widget) -> Self {
        Self {
            handle,
            items: RefCell::new(vec![]),
            image_list: RefCell::new(None),
            next_id: Cell::new(TOOLBAR_ID_BASE),
            next_key: Cell::new(0),
        }
    }

    fn init_control(&self) {
        self.handle
            .send_message(TB_BUTTONSTRUCTSIZE, size_of::<TBBUTTON>(), 0);
        self.handle
            .send_message(TB_SETEXTENDEDSTYLE, 0, TBSTYLE_EX_MIXEDBUTTONS as _);
        let (cx, cy) = small_icon_size(self.handle.dpi());
        self.handle
            .send_message(TB_SETBITMAPSIZE, 0, make_long(cx, cy));
    }

    fn alloc_id(&self) -> u32 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    fn add_item(&self, kind: ItemKind) -> Result<(u32, usize)> {
        let id = if kind == ItemKind::Separator {
            0
        } else {
            self.alloc_id()
        };
        let key = self.next_key.get();
        self.next_key.set(key + 1);
        let i_bitmap = match kind {
            ItemKind::Separator => SEPARATOR_WIDTH,
            _ => I_IMAGENONE,
        };
        let state = if kind == ItemKind::Separator {
            0
        } else {
            TBSTATE_ENABLED as u8
        };
        let button = TBBUTTON {
            iBitmap: i_bitmap,
            idCommand: id as i32,
            fsState: state,
            fsStyle: kind.style(),
            ..Default::default()
        };
        let res = self
            .handle
            .send_message(TB_ADDBUTTONS, 1, &button as *const TBBUTTON as LPARAM);
        if res == 0 {
            return Err(Error::from_thread());
        }
        self.items.borrow_mut().push(ItemState {
            key,
            id,
            icon: None,
            has_text: false,
        });
        self.autosize();
        Ok((id, key))
    }

    fn remove_item(&self, key: usize) {
        let index = self.items.borrow().iter().position(|item| item.key == key);
        let Some(index) = index else {
            return;
        };
        self.handle.send_message(TB_DELETEBUTTON, index, 0);
        self.items.borrow_mut().remove(index);
        let _ = self.rebuild_image_list();
        self.autosize();
    }

    fn autosize(&self) {
        self.handle.send_message(TB_AUTOSIZE, 0, 0);
    }

    fn set_text(&self, id: u32, text: &str) -> Result<()> {
        with_u16c(text, |text| {
            let mut info: TBBUTTONINFOW = unsafe { zeroed() };
            info.cbSize = size_of::<TBBUTTONINFOW>() as _;
            info.dwMask = TBIF_TEXT;
            info.pszText = text.as_ptr() as *mut u16;
            info.cchText = text.len() as _;
            let res = self
                .handle
                .send_message(TB_SETBUTTONINFOW, id as _, &raw mut info as _);
            if res == 0 {
                return Err(Error::from_thread());
            }
            Ok(())
        })?;
        if let Some(item) = self
            .items
            .borrow_mut()
            .iter_mut()
            .find(|item| item.id == id)
        {
            item.has_text = !text.is_empty();
        }
        self.autosize();
        Ok(())
    }

    fn text(&self, id: u32) -> Result<String> {
        let text = self.text_u16(id)?;
        Ok(text.to_string_lossy())
    }

    fn text_u16(&self, id: u32) -> Result<U16CString> {
        let len = self.handle.send_message(TB_GETBUTTONTEXTW, id as WPARAM, 0);
        if len < 0 {
            return Err(Error::from_thread());
        }
        unsafe {
            get_u16c(len as _, |buf| {
                let res = self.handle.send_message(
                    TB_GETBUTTONTEXTW,
                    id as WPARAM,
                    buf.as_mut_ptr() as LPARAM,
                );
                if res < 0 {
                    return Err(Error::from_thread());
                }
                Ok(res as _)
            })
        }
    }

    fn state(&self, id: u32) -> u32 {
        self.handle.send_message(TB_GETSTATE, id as WPARAM, 0) as u32
    }

    fn set_state(&self, id: u32, state: u32) -> Result<()> {
        let res = self
            .handle
            .send_message(TB_SETSTATE, id as WPARAM, state as LPARAM);
        if res == 0 {
            Err(Error::from_thread())
        } else {
            Ok(())
        }
    }

    fn is_enabled(&self, id: u32) -> Result<bool> {
        Ok(self.state(id) & TBSTATE_ENABLED != 0)
    }

    fn set_enabled(&self, id: u32, enabled: bool) -> Result<()> {
        let mut state = self.state(id);
        if enabled {
            state |= TBSTATE_ENABLED;
        } else {
            state &= !TBSTATE_ENABLED;
        }
        self.set_state(id, state)
    }

    fn is_checked(&self, id: u32) -> Result<bool> {
        Ok(self.state(id) & TBSTATE_CHECKED != 0)
    }

    fn set_checked(&self, id: u32, checked: bool) -> Result<()> {
        let mut state = self.state(id);
        if checked {
            state |= TBSTATE_CHECKED;
        } else {
            state &= !TBSTATE_CHECKED;
        }
        self.set_state(id, state)
    }

    fn set_image(&self, id: u32, image: i32) -> Result<()> {
        let mut info: TBBUTTONINFOW = unsafe { zeroed() };
        info.cbSize = size_of::<TBBUTTONINFOW>() as _;
        info.dwMask = TBIF_IMAGE;
        info.iImage = image;
        let res = self.handle.send_message(
            TB_SETBUTTONINFOW,
            id as WPARAM,
            &mut info as *mut TBBUTTONINFOW as LPARAM,
        );
        if res == 0 {
            Err(Error::from_thread())
        } else {
            Ok(())
        }
    }

    fn set_icon(&self, id: u32, icon: Option<&Image>) -> Result<()> {
        let has_text = {
            let mut items = self.items.borrow_mut();
            let Some(item) = items.iter_mut().find(|item| item.id == id) else {
                return Ok(());
            };
            item.icon = icon.cloned();
            item.has_text
        };
        self.rebuild_image_list()?;
        if icon.is_some() && !has_text {
            let dpi = self.handle.dpi();
            let (cx, _) = small_icon_size(dpi);
            self.set_width(id, cx as u16)?;
        }
        Ok(())
    }

    fn set_width(&self, id: u32, cx: u16) -> Result<()> {
        let mut info: TBBUTTONINFOW = unsafe { zeroed() };
        info.cbSize = size_of::<TBBUTTONINFOW>() as _;
        info.dwMask = TBIF_SIZE;
        info.cx = cx;
        let res =
            self.handle
                .send_message(TB_SETBUTTONINFOW, id as WPARAM, &raw mut info as LPARAM);
        if res == 0 {
            Err(Error::from_thread())
        } else {
            Ok(())
        }
    }

    fn rebuild_image_list(&self) -> Result<()> {
        let (cx, cy) = small_icon_size(self.handle.dpi());
        let items = self.items.borrow();
        let has_icons = items.iter().any(|item| item.icon.is_some());

        let list = if has_icons {
            let himl = unsafe { ImageList_Create(cx, cy, ILC_COLOR32, 0, 1) };
            if himl == 0 {
                return Err(Error::from_thread());
            }
            let list = WinImageList(himl);
            let mut indices = Vec::with_capacity(items.len());
            for item in items.iter() {
                let index = match &item.icon {
                    Some(icon) => {
                        let bitmap = icon.to_hbitmap((cx, cy))?;
                        let index = unsafe { ImageList_Add(list.0, bitmap.0, null_mut()) };
                        if index < 0 {
                            return Err(Error::from_thread());
                        }
                        index
                    }
                    None => I_IMAGENONE,
                };
                indices.push((item.id, index));
            }
            self.handle
                .send_message(TB_SETBITMAPSIZE, 0, make_long(cx, cy));
            self.handle
                .send_message(TB_SETIMAGELIST, 0, list.0 as LPARAM);
            for (id, index) in indices {
                if id != 0 {
                    self.set_image(id, index)?;
                }
            }
            Some(list)
        } else {
            self.handle.send_message(TB_SETIMAGELIST, 0, 0);
            for item in items.iter() {
                if item.id != 0 {
                    self.set_image(item.id, I_IMAGENONE)?;
                }
            }
            None
        };

        *self.image_list.borrow_mut() = list;
        drop(items);
        self.autosize();
        Ok(())
    }

    fn is_visible(&self) -> bool {
        self.handle.is_visible().unwrap_or_default()
    }

    fn height(&self) -> i32 {
        let mut rect: RECT = unsafe { zeroed() };
        if unsafe { GetWindowRect(self.handle.as_window().as_win32(), &mut rect) } == 0 {
            0
        } else {
            rect.bottom - rect.top
        }
    }
}

pub(crate) fn visible_toolbar_height(parent: HWND) -> i32 {
    TOOLBARS.with_borrow(|toolbars| {
        toolbars
            .iter()
            .filter(|data| unsafe {
                IsChild(parent, data.handle.as_window().as_win32()) != 0 && data.is_visible()
            })
            .map(|data| data.height())
            .sum()
    })
}

pub(crate) fn refresh_toolbar_icons(root: HWND) -> Result<()> {
    TOOLBARS.with_borrow(|toolbars| {
        for data in toolbars
            .iter()
            .filter(|data| unsafe { IsChild(root, data.handle.as_window().as_win32()) != 0 })
        {
            data.rebuild_image_list()?;
        }
        Ok(())
    })
}
