#![warn(missing_docs)]

use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{
        DWMSBT_AUTO, DWMSBT_MAINWINDOW, DWMSBT_TABBEDWINDOW, DWMSBT_TRANSIENTWINDOW,
        DWMWA_SYSTEMBACKDROP_TYPE, DwmGetWindowAttribute, DwmSetWindowAttribute,
    },
};

/// Backdrop effects for windows.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Backdrop {
    /// Default window style.
    None,
    /// Acrylic effect.
    Acrylic,
    /// Mica effect.
    Mica,
    /// Mica Alt effect.
    MicaAlt,
}

/// Get the current backdrop effect of a window.
/// # Safety
/// The caller must ensure that `handle` is a valid window handle.
pub unsafe fn get_backdrop(handle: HWND) -> Backdrop {
    let mut style = 0;
    let res = unsafe {
        DwmGetWindowAttribute(
            handle,
            DWMWA_SYSTEMBACKDROP_TYPE as _,
            &mut style as *mut _ as _,
            4,
        )
    };
    if res < 0 {
        return Backdrop::None;
    }
    match style {
        DWMSBT_TRANSIENTWINDOW => Backdrop::Acrylic,
        DWMSBT_MAINWINDOW => Backdrop::Mica,
        DWMSBT_TABBEDWINDOW => Backdrop::MicaAlt,
        _ => Backdrop::None,
    }
}

/// Set the backdrop effect of a window.
/// # Safety
/// The caller must ensure that `handle` is a valid window handle.
pub unsafe fn set_backdrop(handle: HWND, backdrop: Backdrop) -> bool {
    let style = match backdrop {
        Backdrop::Acrylic => DWMSBT_TRANSIENTWINDOW,
        Backdrop::Mica => DWMSBT_MAINWINDOW,
        Backdrop::MicaAlt => DWMSBT_TABBEDWINDOW,
        _ => DWMSBT_AUTO,
    };
    let res = DwmSetWindowAttribute(
        handle,
        DWMWA_SYSTEMBACKDROP_TYPE as _,
        &style as *const _ as _,
        4,
    );
    res >= 0 && style > 0
}
