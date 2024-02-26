#[cfg(feature = "lazy_cell")]
use std::sync::LazyLock;
use std::{collections::BTreeMap, io, sync::Mutex};

use compio::driver::syscall;
#[cfg(not(feature = "lazy_cell"))]
use once_cell::sync::Lazy as LazyLock;
use windows_sys::Win32::{
    Graphics::Gdi::{CreateFontIndirectW, DeleteObject, HFONT, LOGFONTW},
    UI::{
        HiDpi::SystemParametersInfoForDpi,
        WindowsAndMessaging::{
            NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, USER_DEFAULT_SCREEN_DPI,
        },
    },
};

use crate::ui::dpi::DpiAware;

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

struct WinFont(HFONT);

impl Drop for WinFont {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.0) };
    }
}

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
