#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;
use std::{cell::RefCell, collections::BTreeMap, mem::MaybeUninit, ptr::null_mut, sync::Mutex};

#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;
use widestring::{U16CStr, U16Str};
use windows_sys::Win32::{
    Foundation::{HWND, SIZE},
    Graphics::Gdi::{
        CreateCompatibleDC, CreateFontIndirectW, DeleteDC, DeleteObject, GetObjectW,
        GetTextExtentPoint32W, GetTextMetricsW, HDC, HFONT, LOGFONTW, SelectObject, TEXTMETRICW,
    },
    UI::{
        HiDpi::SystemParametersInfoForDpi,
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
    ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
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

struct MeasureDc(HDC);

impl Drop for MeasureDc {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.0);
        }
    }
}

thread_local! {
    static MEASURE_DC: MeasureDc = MeasureDc(unsafe { CreateCompatibleDC(null_mut()) });
}

pub fn measure_string(hwnd: HWND, s: &U16Str) -> Result<Size> {
    unsafe {
        let dpi = get_dpi_for_window(hwnd);
        let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0) as HFONT;
        let hfont = if hfont.is_null() {
            default_font(dpi)?
        } else {
            hfont
        };
        MEASURE_DC.with(|dc| {
            let hdc = dc.0;
            if hdc.is_null() {
                return Err(Error::from_thread());
            }
            let old = SelectObject(hdc, hfont);
            let measured = measure_with_dc(hdc, s);
            SelectObject(hdc, old);
            Ok(measured?.to_logical(dpi))
        })
    }
}

unsafe fn measure_with_dc(hdc: HDC, s: &U16Str) -> Result<Size> {
    let mut metrics = MaybeUninit::<TEXTMETRICW>::uninit();
    if unsafe { GetTextMetricsW(hdc, metrics.as_mut_ptr()) } == 0 {
        return Err(Error::from_thread());
    }
    let metrics = unsafe { metrics.assume_init() };
    let line_height = metrics.tmHeight + metrics.tmExternalLeading;
    let mut width = 0;
    let mut lines = 0;
    for line in s.as_slice().split(|&c| c == b'\n' as u16) {
        let line = line.strip_suffix(&[b'\r' as u16]).unwrap_or(line);
        let mut size: SIZE = unsafe { std::mem::zeroed() };
        unsafe { GetTextExtentPoint32W(hdc, line.as_ptr(), line.len() as _, &mut size) };
        width = width.max(size.cx);
        lines += 1;
    }
    Ok(Size::new(width as _, (line_height * lines) as _))
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
        for slot in f.lfFaceName.iter_mut() {
            *slot = chars.next().unwrap_or(0);
        }
    })
}

/// Read a [`Font`] from an [`HFONT`], scaled back to logical size by the DPI
/// of the window.
pub(crate) fn hfont_to_font(hwnd: HWND, hfont: HFONT) -> Result<Font> {
    let mut lf = MaybeUninit::<LOGFONTW>::uninit();
    let size = unsafe { GetObjectW(hfont, size_of::<LOGFONTW>() as _, lf.as_mut_ptr().cast()) };
    if size == 0 {
        return Err(Error::from_thread());
    }
    let lf = unsafe { lf.assume_init() };
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
