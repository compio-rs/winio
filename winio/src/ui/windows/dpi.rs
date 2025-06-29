use windows_sys::Win32::{
    Foundation::HWND,
    System::WindowsProgramming::MulDiv,
    UI::{
        HiDpi::{GetDpiForSystem, GetDpiForWindow},
        WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI,
    },
};

use crate::{Point, Rect, Size};

#[inline]
pub unsafe fn get_dpi_for_window(h_wnd: HWND) -> u32 {
    if !h_wnd.is_null() {
        GetDpiForWindow(h_wnd)
    } else {
        GetDpiForSystem()
    }
}

pub trait DpiAware {
    fn to_logical(self, dpi: u32) -> Self;
    fn to_device(self, dpi: u32) -> Self;
}

impl DpiAware for i32 {
    #[inline]
    fn to_logical(self, dpi: u32) -> Self {
        unsafe { MulDiv(self, USER_DEFAULT_SCREEN_DPI as _, dpi as i32) }
    }

    #[inline]
    fn to_device(self, dpi: u32) -> Self {
        unsafe { MulDiv(self, dpi as i32, USER_DEFAULT_SCREEN_DPI as _) }
    }
}

macro_rules! impl_dpi_aware_for_f64 {
    ($t:ty) => {
        impl DpiAware for $t {
            #[inline]
            fn to_logical(self, dpi: u32) -> Self {
                self / (dpi as f64) * (USER_DEFAULT_SCREEN_DPI as f64)
            }

            #[inline]
            fn to_device(self, dpi: u32) -> Self {
                self / (USER_DEFAULT_SCREEN_DPI as f64) * (dpi as f64)
            }
        }
    };
}

impl_dpi_aware_for_f64!(f64);
impl_dpi_aware_for_f64!(Point);
impl_dpi_aware_for_f64!(Size);
impl_dpi_aware_for_f64!(Rect);
