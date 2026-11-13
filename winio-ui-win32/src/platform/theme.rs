use std::{mem::MaybeUninit, ptr::null_mut};

use windows_sys::Win32::{
    Foundation::{HWND, RECT, SIZE},
    UI::{
        Controls::{
            CloseThemeData, GetThemeBackgroundContentRect, GetThemeMargins, GetThemePartSize,
            HTHEME, MARGINS, TS_TRUE,
        },
        HiDpi::OpenThemeDataForDpi,
        WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI,
    },
};
use winio_primitive::{Margin, Point, Rect, Size};

/// A visual style handle, opened for the reference DPI.
#[derive(Debug)]
pub(crate) struct Theme(HTHEME);

impl Theme {
    pub fn open(hwnd: HWND, class: *const u16) -> Option<Self> {
        let htheme = unsafe { OpenThemeDataForDpi(hwnd, class, USER_DEFAULT_SCREEN_DPI) };
        (htheme != 0).then_some(Self(htheme))
    }

    pub fn part_size(&self, part: i32, state: i32) -> Option<Size> {
        let mut size = MaybeUninit::<SIZE>::uninit();
        let hr = unsafe {
            GetThemePartSize(
                self.0,
                null_mut(),
                part,
                state,
                std::ptr::null(),
                TS_TRUE,
                size.as_mut_ptr(),
            )
        };
        if hr < 0 {
            return None;
        }
        let size = unsafe { size.assume_init() };
        Some(Size::new(size.cx as _, size.cy as _))
    }

    pub fn margins(&self, part: i32, state: i32, prop: u32) -> Option<Margin> {
        let mut margins = MaybeUninit::<MARGINS>::uninit();
        let hr = unsafe {
            GetThemeMargins(
                self.0,
                null_mut(),
                part,
                state,
                prop as _,
                std::ptr::null(),
                margins.as_mut_ptr(),
            )
        };
        if hr < 0 {
            return None;
        }
        let m = unsafe { margins.assume_init() };
        Some(Margin::new(
            m.cyTopHeight as _,
            m.cxRightWidth as _,
            m.cyBottomHeight as _,
            m.cxLeftWidth as _,
        ))
    }

    /// Shrink a background rectangle into the content rectangle inside it.
    pub fn content_rect(&self, part: i32, state: i32, bounds: Rect) -> Option<Rect> {
        let bounds = rect_to_win32(bounds);
        let mut content = MaybeUninit::<RECT>::uninit();
        let hr = unsafe {
            GetThemeBackgroundContentRect(
                self.0,
                null_mut(),
                part,
                state,
                &bounds,
                content.as_mut_ptr(),
            )
        };
        if hr < 0 {
            return None;
        }
        let content = unsafe { content.assume_init() };
        Some(rect_from_win32(content))
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe {
            CloseThemeData(self.0);
        }
    }
}

fn rect_to_win32(r: Rect) -> RECT {
    RECT {
        left: r.origin.x as _,
        top: r.origin.y as _,
        right: (r.origin.x + r.size.width) as _,
        bottom: (r.origin.y + r.size.height) as _,
    }
}

fn rect_from_win32(r: RECT) -> Rect {
    Rect::new(
        Point::new(r.left as _, r.top as _),
        Size::new((r.right - r.left) as _, (r.bottom - r.top) as _),
    )
}
