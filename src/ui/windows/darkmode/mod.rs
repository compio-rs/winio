use widestring::U16CStr;
use windows_sys::Win32::{
    Foundation::{COLORREF, HWND, LRESULT},
    Globalization::{CSTR_EQUAL, CompareStringW, LOCALE_ALL, NORM_IGNORECASE},
    Graphics::Gdi::{
        BLACK_BRUSH, GetStockObject, HDC, NULL_BRUSH, SetBkColor, SetBkMode, SetTextColor,
        TRANSPARENT, WHITE_BRUSH,
    },
    System::SystemServices::MAX_CLASS_NAME,
    UI::{Controls::WC_STATICW, WindowsAndMessaging::GetClassNameW},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "windows-dark-mode")] {
        #[path = "hook.rs"]
        mod imp;
        pub use imp::*;
    } else {
        #[path = "fallback.rs"]
        mod imp;
        pub use imp::*;
    }
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

#[inline]
fn u16_string_eq_ignore_case(s1: &U16CStr, s2: *const u16) -> bool {
    unsafe {
        CompareStringW(
            LOCALE_ALL,
            NORM_IGNORECASE,
            s1.as_ptr(),
            s1.len() as _,
            s2,
            -1,
        ) == CSTR_EQUAL
    }
}

const WHITE: COLORREF = 0x00FFFFFF;
const BLACK: COLORREF = 0x00000000;

pub unsafe fn control_color_static(hwnd: HWND, hdc: HDC) -> LRESULT {
    let dark = is_dark_mode_allowed_for_app();

    let mut class = [0u16; MAX_CLASS_NAME as usize];
    GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
    let class = U16CStr::from_ptr_str(class.as_ptr());
    let is_static = u16_string_eq_ignore_case(class, WC_STATICW);

    SetBkMode(hdc, TRANSPARENT as _);
    if dark {
        SetTextColor(hdc, WHITE);
        if !is_static {
            SetBkColor(hdc, BLACK);
        }
    }
    let res = if is_static {
        GetStockObject(NULL_BRUSH)
    } else if dark {
        GetStockObject(BLACK_BRUSH)
    } else {
        GetStockObject(WHITE_BRUSH)
    };
    res as _
}
