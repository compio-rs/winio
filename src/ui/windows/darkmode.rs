use std::{
    mem::MaybeUninit,
    ptr::{null, null_mut},
};

use widestring::{U16CStr, U16CString};
use windows_sys::{
    Win32::{
        Foundation::{BOOL, BOOLEAN, COLORREF, HWND, LPARAM, S_OK},
        Globalization::{CSTR_EQUAL, CompareStringW, LOCALE_ALL, NORM_IGNORECASE},
        Graphics::{
            Dwm::DwmSetWindowAttribute,
            Gdi::{
                CreateCompatibleBitmap, CreateCompatibleDC, DT_CALCRECT, DT_HIDEPREFIX, DeleteDC,
                DeleteObject, DrawTextW, GetDC, HGDIOBJ, InvalidateRect, ReleaseDC, SelectObject,
                SetBkColor, SetBkMode, SetTextColor, TRANSPARENT,
            },
        },
        System::SystemServices::MAX_CLASS_NAME,
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Controls::{PROGRESS_CLASSW, SetWindowTheme, WC_BUTTONW, WC_COMBOBOXW},
            WindowsAndMessaging::{
                BM_SETIMAGE, BS_BITMAP, BS_DEFPUSHBUTTON, BS_OWNERDRAW, BS_TYPEMASK,
                EnumChildWindows, GWL_STYLE, GetClassNameW, GetClientRect, GetWindowLongPtrW,
                GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, IMAGE_BITMAP,
                SPI_GETHIGHCONTRAST, SendMessageW, SetWindowLongPtrW, SetWindowLongW,
                SystemParametersInfoW, WM_GETFONT,
            },
        },
    },
    core::HRESULT,
    w,
};

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetNtVersionNumbers(major: *mut u32, minor: *mut u32, build: *mut u32);
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum PreferredAppMode {
    Default    = 0,
    AllowDark  = 1,
    ForceDark  = 2,
    ForceLight = 3,
}

#[link(name = "uxtheme", kind = "raw-dylib")]
extern "system" {
    #[link_ordinal(132)]
    fn ShouldAppsUseDarkMode() -> BOOLEAN;
    #[link_ordinal(133)]
    fn AllowDarkModeForWindow(h: HWND, a: BOOLEAN) -> BOOLEAN;
    // build < 18362
    #[link_ordinal(135)]
    fn AllowDarkModeForApp(a: BOOLEAN) -> BOOLEAN;
    // build >= 18362
    #[link_ordinal(135)]
    fn SetPreferredAppMode(m: PreferredAppMode) -> PreferredAppMode;
    #[link_ordinal(136)]
    fn FlushMenuThemes();
}

#[inline]
unsafe fn get_nt_build() -> u32 {
    let mut build = 0;
    RtlGetNtVersionNumbers(null_mut(), null_mut(), &mut build);
    build &= !0xF0000000;
    build
}

pub unsafe fn set_preferred_app_mode(m: PreferredAppMode) -> PreferredAppMode {
    let build = get_nt_build();
    if build < 18362 {
        if AllowDarkModeForApp(
            if m == PreferredAppMode::AllowDark || m == PreferredAppMode::ForceDark {
                1
            } else {
                0
            },
        ) != 0
        {
            PreferredAppMode::AllowDark
        } else {
            PreferredAppMode::Default
        }
    } else {
        SetPreferredAppMode(m)
    }
}

pub unsafe fn is_dark_mode_allowed_for_app() -> bool {
    let mut hc: HIGHCONTRASTW = std::mem::zeroed();
    hc.cbSize = size_of::<HIGHCONTRASTW>() as u32;
    if SystemParametersInfoW(
        SPI_GETHIGHCONTRAST,
        hc.cbSize,
        std::ptr::addr_of_mut!(hc).cast(),
        0,
    ) == 0
    {
        return false;
    }
    ((hc.dwFlags & HCF_HIGHCONTRASTON) == 0) && (ShouldAppsUseDarkMode() != 0)
}

const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 0x13;
const DWMWA_USE_IMMERSIVE_DARK_MODE_V2: u32 = 0x14;

pub unsafe fn window_use_dark_mode(h_wnd: HWND) -> HRESULT {
    let set_dark_mode = is_dark_mode_allowed_for_app() as BOOL;
    AllowDarkModeForWindow(h_wnd, set_dark_mode as BOOLEAN);
    let hr = DwmSetWindowAttribute(
        h_wnd,
        DWMWA_USE_IMMERSIVE_DARK_MODE_V2,
        std::ptr::addr_of!(set_dark_mode).cast(),
        size_of::<BOOL>() as _,
    );
    if hr != 0 {
        let hr = DwmSetWindowAttribute(
            h_wnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            std::ptr::addr_of!(set_dark_mode).cast(),
            size_of::<BOOL>() as _,
        );
        if hr != 0 {
            return hr;
        }
    }
    FlushMenuThemes();
    S_OK
}

pub unsafe fn children_refresh_dark_mode(handle: HWND) {
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        control_use_dark_mode(hwnd);
        InvalidateRect(hwnd, null(), 1);
        EnumChildWindows(hwnd, Some(enum_callback), lparam);
        1
    }

    EnumChildWindows(handle, Some(enum_callback), 0);
}

#[inline]
fn u16_string_eq_ignore_case(s1: &U16CStr, s2: *const u16) -> bool {
    unsafe { CompareStringW(LOCALE_ALL, NORM_IGNORECASE, s1.as_ptr(), -1, s2, -1) == CSTR_EQUAL }
}

pub unsafe fn control_use_dark_mode(hwnd: HWND) {
    let mut class = [0u16; MAX_CLASS_NAME as usize];
    GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
    let class = U16CStr::from_ptr_str(class.as_ptr());
    if is_dark_mode_allowed_for_app() {
        let subappname = if u16_string_eq_ignore_case(class, WC_COMBOBOXW) {
            w!("DarkMode_CFD")
        } else if u16_string_eq_ignore_case(class, PROGRESS_CLASSW) {
            null()
        } else {
            w!("DarkMode_Explorer")
        };
        SetWindowTheme(hwnd, subappname, null());
    } else {
        SetWindowTheme(hwnd, null(), null());
    }
    if u16_string_eq_ignore_case(class, WC_BUTTONW) {
        fix_button_dark_mode(hwnd);
    }
}

pub unsafe fn fix_button_dark_mode(hwnd: HWND) {
    let style = if cfg!(target_pointer_width = "64") {
        GetWindowLongPtrW(hwnd, GWL_STYLE) as i32
    } else {
        GetWindowLongW(hwnd, GWL_STYLE) as i32
    };
    let button_type = style & BS_TYPEMASK;
    if button_type <= BS_DEFPUSHBUTTON || button_type >= BS_OWNERDRAW {
        return;
    }
    let len = GetWindowTextLengthW(hwnd);
    if is_dark_mode_allowed_for_app() && len > 0 {
        let mut res: Vec<u16> = Vec::with_capacity(len as usize + 1);
        GetWindowTextW(hwnd, res.as_mut_ptr(), res.capacity() as _);
        res.set_len(len as usize + 1);
        let text = U16CString::from_vec_unchecked(res);
        let hdc = GetDC(hwnd);
        if !hdc.is_null() {
            let font = SendMessageW(hwnd, WM_GETFONT, 0, 0) as HGDIOBJ;
            let oldfont = SelectObject(hdc, font);
            let mut rc = MaybeUninit::uninit();
            GetClientRect(hwnd, rc.as_mut_ptr());
            let mut rc = rc.assume_init();
            DrawTextW(
                hdc,
                text.as_ptr(),
                text.len() as i32 + 1,
                &mut rc,
                DT_HIDEPREFIX | DT_CALCRECT,
            );
            let hbm = CreateCompatibleBitmap(hdc, rc.right - rc.left, rc.bottom - rc.top);
            let hmdc = CreateCompatibleDC(hdc);
            let hold = SelectObject(hmdc, hbm);
            let old_font_m = SelectObject(hmdc, font);
            SetBkMode(hmdc, TRANSPARENT as _);
            SetTextColor(hmdc, WHITE);
            SetBkColor(hmdc, BLACK);
            DrawTextW(
                hmdc,
                text.as_ptr(),
                text.len() as i32 + 1,
                &mut rc,
                DT_HIDEPREFIX,
            );
            SelectObject(hmdc, old_font_m);
            SelectObject(hmdc, hold);
            DeleteDC(hmdc);
            SelectObject(hdc, oldfont);
            ReleaseDC(hwnd, hdc);
            let style = style | BS_BITMAP;
            if cfg!(target_pointer_width = "64") {
                SetWindowLongPtrW(hwnd, GWL_STYLE, style as _);
            } else {
                SetWindowLongW(hwnd, GWL_STYLE, style as _);
            }
            let oldbm = SendMessageW(hwnd, BM_SETIMAGE, IMAGE_BITMAP as _, hbm as _);
            if oldbm != 0 {
                DeleteObject(oldbm as _);
            }
        }
    } else {
        let oldbm = SendMessageW(hwnd, BM_SETIMAGE, IMAGE_BITMAP as _, 0);
        if oldbm != 0 {
            DeleteObject(oldbm as _);
        }
        let style = style & !BS_BITMAP;
        if cfg!(target_pointer_width = "64") {
            SetWindowLongPtrW(hwnd, GWL_STYLE, style as _);
        } else {
            SetWindowLongW(hwnd, GWL_STYLE, style as _);
        }
    }
}

const WHITE: COLORREF = 0x00FFFFFF;
const BLACK: COLORREF = 0x00000000;
