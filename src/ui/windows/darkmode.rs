use std::ptr::null_mut;

use windows_sys::{
    Win32::{
        Foundation::{BOOL, BOOLEAN, HWND, S_OK},
        Graphics::Dwm::DwmSetWindowAttribute,
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Controls::SetWindowTheme,
            WindowsAndMessaging::{SPI_GETHIGHCONTRAST, SystemParametersInfoW},
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

pub unsafe fn control_use_dark_mode(h_wnd: HWND) -> HRESULT {
    if is_dark_mode_allowed_for_app() {
        SetWindowTheme(h_wnd, w!("DarkMode_Explorer"), null_mut())
    } else {
        S_OK
    }
}
