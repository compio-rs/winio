use core::f32;
#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;
use std::{collections::BTreeMap, io, mem::MaybeUninit, sync::Mutex};

use compio::driver::syscall;
#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;
use widestring::U16Str;
use windows::{
    Win32::Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_ITALIC,
        DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWriteCreateFactory, IDWriteFactory,
    },
    core::w,
};
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{CreateFontIndirectW, DeleteObject, GetObjectW, HFONT, LOGFONTW},
    UI::{
        HiDpi::{GetDpiForWindow, SystemParametersInfoForDpi},
        WindowsAndMessaging::{
            NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SendMessageW, USER_DEFAULT_SCREEN_DPI,
            WM_GETFONT,
        },
    },
};
use winio_primitive::Size;

use super::dpi::DpiAware;
use crate::Result;

unsafe fn system_default_font() -> Result<LOGFONTW> {
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

static DEFAULT_FONT: OnceLock<LOGFONTW> = OnceLock::new();

fn default_log_font() -> Result<&'static LOGFONTW> {
    DEFAULT_FONT.get_or_try_init(|| unsafe { system_default_font() })
}

static DPI_FONTS: Mutex<BTreeMap<u32, WinFont>> = Mutex::new(BTreeMap::new());

pub fn default_font(dpi: u32) -> Result<HFONT> {
    let mut map = DPI_FONTS.lock().unwrap();
    match map.get(&dpi) {
        Some(f) => Ok(f.0),
        None => unsafe {
            let mut f = *default_log_font()?;
            f.lfHeight = f.lfHeight.to_device(dpi);
            f.lfWidth = f.lfWidth.to_device(dpi);
            let res = CreateFontIndirectW(&f);
            if res.is_null() {
                return Err(io::Error::last_os_error().into());
            }
            map.insert(dpi, WinFont(res));
            Ok(res)
        },
    }
}

static DWRITE_FACTORY: OnceLock<IDWriteFactory> = OnceLock::new();

pub fn dwrite_factory() -> Result<&'static IDWriteFactory> {
    DWRITE_FACTORY.get_or_try_init(|| unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) })
}

pub fn measure_string(hwnd: HWND, s: &U16Str) -> Result<Size> {
    unsafe {
        let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0) as HFONT;
        let mut font = MaybeUninit::<LOGFONTW>::uninit();
        if GetObjectW(
            hfont,
            std::mem::size_of::<LOGFONTW>() as _,
            font.as_mut_ptr().cast(),
        ) == 0
        {
            return Ok(Size::zero());
        }
        let font = font.assume_init();
        let dpi = GetDpiForWindow(hwnd);
        let height = font.lfHeight.abs().to_logical(dpi);
        if s.is_empty() {
            return Ok(Size::new(0.0, height as _));
        }

        let factory = dwrite_factory()?;
        let format = factory.CreateTextFormat(
            windows::core::PCWSTR::from_raw(font.lfFaceName.as_ptr()),
            None,
            DWRITE_FONT_WEIGHT(font.lfWeight),
            if font.lfItalic != 0 {
                DWRITE_FONT_STYLE_ITALIC
            } else {
                DWRITE_FONT_STYLE_NORMAL
            },
            DWRITE_FONT_STRETCH_NORMAL,
            height as f32,
            w!(""),
        )?;
        let layout = factory.CreateTextLayout(s.as_slice(), &format, f32::MAX, f32::MAX)?;
        let mut metrics = MaybeUninit::uninit();
        layout.GetMetrics(metrics.as_mut_ptr())?;
        let metrics = metrics.assume_init();
        Ok(Size::new(metrics.width as _, metrics.height as _))
    }
}
