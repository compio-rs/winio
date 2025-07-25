use core::f32;
use std::{
    collections::BTreeMap,
    io,
    mem::MaybeUninit,
    sync::{LazyLock, Mutex},
};

use compio::driver::syscall;
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

pub static DWRITE_FACTORY: LazyLock<IDWriteFactory> =
    LazyLock::new(|| unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap() });

pub fn measure_string(hwnd: HWND, s: &U16Str) -> Size {
    unsafe {
        let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0) as HFONT;
        let mut font = MaybeUninit::<LOGFONTW>::uninit();
        if GetObjectW(
            hfont,
            std::mem::size_of::<LOGFONTW>() as _,
            font.as_mut_ptr().cast(),
        ) == 0
        {
            return Size::zero();
        }
        let font = font.assume_init();
        let dpi = GetDpiForWindow(hwnd);
        let height = font.lfHeight.abs().to_logical(dpi);
        if s.is_empty() {
            return Size::new(0.0, height as _);
        }

        let format = DWRITE_FACTORY
            .CreateTextFormat(
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
            )
            .unwrap();
        let layout = DWRITE_FACTORY
            .CreateTextLayout(s.as_slice(), &format, f32::MAX, f32::MAX)
            .unwrap();
        let mut metrics = MaybeUninit::uninit();
        layout.GetMetrics(metrics.as_mut_ptr()).unwrap();
        let metrics = metrics.assume_init();
        Size::new(metrics.width as _, metrics.height as _)
    }
}
