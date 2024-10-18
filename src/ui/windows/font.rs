use std::{
    collections::BTreeMap,
    io,
    mem::MaybeUninit,
    sync::{LazyLock, Mutex},
};

use compio::driver::syscall;
use widestring::U16CStr;
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        CreateFontIndirectW, DeleteObject, GetTextExtentPoint32W, GetWindowDC, HDC, HFONT, HGDIOBJ,
        LOGFONTW, ReleaseDC,
    },
    UI::{
        HiDpi::SystemParametersInfoForDpi,
        WindowsAndMessaging::{
            NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, USER_DEFAULT_SCREEN_DPI,
        },
    },
};

use super::dpi::DpiAware;
use crate::Size;

unsafe fn system_default_font() -> io::Result<LOGFONTW> {
    let mut ncm: NONCLIENTMETRICSW = unsafe { std::mem::zeroed() };
    ncm.cbSize = std::mem::size_of::<NONCLIENTMETRICSW>() as u32;
    syscall!(
        BOOL,
        SystemParametersInfoForDpi(
            SPI_GETNONCLIENTMETRICS,
            ncm.cbSize,
            &mut ncm as *mut _ as _,
            0,
            USER_DEFAULT_SCREEN_DPI as _,
        )
    )?;
    Ok(ncm.lfMessageFont)
}

pub(crate) struct WinFont(pub HFONT);

impl Drop for WinFont {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.0) };
    }
}

unsafe impl Send for WinFont {}
unsafe impl Sync for WinFont {}

static DEFAULT_FONT: LazyLock<LOGFONTW> =
    LazyLock::new(|| unsafe { system_default_font() }.unwrap());
static DPI_FONTS: Mutex<BTreeMap<u32, WinFont>> = Mutex::new(BTreeMap::new());

pub fn default_font(dpi: u32) -> HFONT {
    let mut map = DPI_FONTS.lock().unwrap();
    match map.get(&dpi) {
        Some(f) => f.0,
        None => unsafe {
            let mut f = *DEFAULT_FONT;
            f.lfHeight = f.lfHeight.to_device(dpi);
            f.lfWidth = f.lfWidth.to_device(dpi);
            let res = CreateFontIndirectW(&f);
            map.insert(dpi, WinFont(res));
            res
        },
    }
}

pub(crate) struct WinDC(pub HWND, pub HDC);

impl WinDC {
    pub fn new(hwnd: HWND) -> Self {
        unsafe {
            let hdc = GetWindowDC(hwnd);
            Self(hwnd, hdc)
        }
    }
}

impl Drop for WinDC {
    fn drop(&mut self) {
        if !self.1.is_null() {
            unsafe { ReleaseDC(self.0, self.1) };
        }
    }
}

pub(crate) struct WinBrush(pub HGDIOBJ);

impl Drop for WinBrush {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.0) };
    }
}

unsafe impl Send for WinBrush {}
unsafe impl Sync for WinBrush {}

pub fn measure_string(hwnd: HWND, s: &U16CStr) -> Size {
    if s.is_empty() {
        return Size::zero();
    }
    let hdc = WinDC::new(hwnd);
    if hdc.1.is_null() {
        return Size::zero();
    }
    let mut size = MaybeUninit::uninit();
    syscall!(
        BOOL,
        GetTextExtentPoint32W(hdc.1, s.as_ptr(), s.len() as _, size.as_mut_ptr())
    )
    .unwrap();
    let size = unsafe { size.assume_init() };
    Size::new(size.cx as _, size.cy as _)
}
