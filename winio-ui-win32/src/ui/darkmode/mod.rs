use std::{mem::MaybeUninit, sync::LazyLock};

use widestring::U16CStr;
use windows_sys::Win32::{
    Foundation::{COLORREF, HWND, LRESULT},
    Globalization::{CSTR_EQUAL, CompareStringW, LOCALE_ALL, NORM_IGNORECASE},
    Graphics::Gdi::{
        BLACK_BRUSH, CreateSolidBrush, GetStockObject, HDC, NULL_BRUSH, ScreenToClient, SetBkColor,
        SetBkMode, SetTextColor, TRANSPARENT, WHITE_BRUSH,
    },
    System::SystemServices::MAX_CLASS_NAME,
    UI::{
        Controls::{WC_EDITW, WC_STATICW},
        WindowsAndMessaging::{
            ChildWindowFromPoint, GWL_EXSTYLE, GetClassNameW, GetCursorPos, GetWindowLongPtrW,
            WS_EX_TRANSPARENT,
        },
    },
};

use crate::ui::font::WinBrush;

cfg_if::cfg_if! {
    if #[cfg(feature = "dark-mode")] {
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

static EDIT_NORMAL_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(0x00212121) }));

pub unsafe fn control_color_static(hwnd: HWND, hdc: HDC) -> LRESULT {
    let dark = is_dark_mode_allowed_for_app();

    let mut class = [0u16; MAX_CLASS_NAME as usize];
    GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
    let class = U16CStr::from_ptr_str(class.as_ptr());
    let is_static = u16_string_eq_ignore_case(class, WC_STATICW)
        && ((GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TRANSPARENT) != 0);

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

pub unsafe fn control_color_edit(hparent: HWND, hwnd: HWND, hdc: HDC) -> Option<LRESULT> {
    if is_dark_mode_allowed_for_app() {
        let mut class = [0u16; MAX_CLASS_NAME as usize];
        GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
        let class = U16CStr::from_ptr_str(class.as_ptr());

        SetTextColor(hdc, WHITE);
        SetBkColor(hdc, BLACK);
        let is_hover = if u16_string_eq_ignore_case(class, WC_EDITW) {
            let mut p = MaybeUninit::uninit();
            GetCursorPos(p.as_mut_ptr());
            let mut p = p.assume_init();
            ScreenToClient(hwnd, &mut p);
            std::ptr::eq(hwnd, ChildWindowFromPoint(hparent, p))
        } else {
            false
        };
        Some(if is_hover {
            GetStockObject(BLACK_BRUSH)
        } else {
            EDIT_NORMAL_BACK.0
        } as _)
    } else {
        None
    }
}
