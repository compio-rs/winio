#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;
use std::{cell::RefCell, collections::BTreeMap, mem::MaybeUninit, sync::Mutex};

#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;
use widestring::{U16CStr, U16Str};
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
use winio_primitive::{Font, Size};
use winio_ui_windows_common::syscall;

use super::dpi::{DpiAware, get_dpi_for_window};
use crate::{Error, Result};

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

fn create_font(dpi: u32, custom: impl FnOnce(&mut LOGFONTW)) -> Result<WinFont> {
    let mut f = *default_log_font()?;
    f.lfHeight = f.lfHeight.to_device(dpi);
    f.lfWidth = f.lfWidth.to_device(dpi);
    custom(&mut f);
    unsafe {
        let res = CreateFontIndirectW(&f);
        if res.is_null() {
            Err(Error::from_thread())
        } else {
            Ok(WinFont(res))
        }
    }
}

static DPI_FONTS: Mutex<BTreeMap<u32, WinFont>> = Mutex::new(BTreeMap::new());

pub fn default_font(dpi: u32) -> Result<HFONT> {
    let mut map = DPI_FONTS.lock().unwrap();
    match map.get(&dpi) {
        Some(f) => Ok(f.0),
        None => {
            let font = create_font(dpi, |_| {})?;
            let res = font.0;
            map.insert(dpi, font);
            Ok(res)
        }
    }
}

static DPI_UNDERLINE_FONTS: Mutex<BTreeMap<u32, WinFont>> = Mutex::new(BTreeMap::new());

pub fn default_underline_font(dpi: u32) -> Result<HFONT> {
    let mut map = DPI_UNDERLINE_FONTS.lock().unwrap();
    match map.get(&dpi) {
        Some(f) => Ok(f.0),
        None => {
            let font = create_font(dpi, |f| {
                f.lfUnderline = 1;
            })?;
            let res = font.0;
            map.insert(dpi, font);
            Ok(res)
        }
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

thread_local! {
    static LABEL_FONTS: RefCell<BTreeMap<HWND, (Font, WinFont)>> = const { RefCell::new(BTreeMap::new()) };
}

/// Create an [`HFONT`] from a [`Font`], scaled to the DPI of the window.
pub(crate) fn font_to_hfont(hwnd: HWND, font: &Font, underline: bool) -> Result<WinFont> {
    let dpi = get_dpi_for_window(hwnd);
    create_font(dpi, |f| {
        f.lfHeight = -(font.size.to_device(dpi) as i32);
        f.lfWeight = if font.bold { 700 } else { 400 };
        f.lfItalic = font.italic as u8;
        if underline {
            f.lfUnderline = 1;
        }
        let mut chars = font.family.encode_utf16();
        for slot in f.lfFaceName.iter_mut().take(31) {
            *slot = chars.next().unwrap_or(0);
        }
        f.lfFaceName[31] = 0;
    })
}

/// Read a [`Font`] from an [`HFONT`], scaled back to logical size by the DPI
/// of the window.
pub(crate) fn hfont_to_font(hwnd: HWND, hfont: HFONT) -> Result<Font> {
    let mut lf: LOGFONTW = unsafe { std::mem::zeroed() };
    let size = unsafe {
        GetObjectW(
            hfont,
            std::mem::size_of::<LOGFONTW>() as _,
            &mut lf as *mut _ as _,
        )
    };
    if size == 0 {
        return Err(Error::from_thread());
    }
    let dpi = get_dpi_for_window(hwnd);
    let family = unsafe { U16CStr::from_ptr_str(lf.lfFaceName.as_ptr()) }.to_string_lossy();
    Ok(Font {
        family,
        size: (lf.lfHeight.abs() as f64).to_logical(dpi),
        bold: lf.lfWeight >= 600,
        italic: lf.lfItalic != 0,
    })
}

/// Store the font of a window, and return the created [`HFONT`].
pub(crate) fn set_hwnd_font(hwnd: HWND, font: Font, underline: bool) -> Result<HFONT> {
    let hfont = font_to_hfont(hwnd, &font, underline)?;
    let res = hfont.0;
    LABEL_FONTS.with(|map| {
        map.borrow_mut().insert(hwnd, (font, hfont));
    });
    Ok(res)
}

/// Recreate the stored font of a window for its current DPI, and return the
/// new [`HFONT`], or `None` if no font is stored for the window.
pub(crate) fn refresh_hwnd_font(hwnd: HWND, underline: bool) -> Result<Option<HFONT>> {
    let font = match LABEL_FONTS.with(|map| map.borrow().get(&hwnd).map(|(f, _)| f.clone())) {
        Some(x) => x,
        None => return Ok(None),
    };
    let hfont = font_to_hfont(hwnd, &font, underline)?;
    let res = hfont.0;
    LABEL_FONTS.with(|map| {
        map.borrow_mut().insert(hwnd, (font, hfont));
    });
    Ok(Some(res))
}

/// Remove the stored font of a window.
pub(crate) fn remove_hwnd_font(hwnd: HWND) {
    LABEL_FONTS.with(|map| {
        map.borrow_mut().remove(&hwnd);
    });
}
