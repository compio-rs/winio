use std::mem::MaybeUninit;

use widestring::{U16CStr, U16CString};
use winio_primitive::ColorTheme;

pub(crate) mod darkmode;
pub(crate) mod dpi;
pub(crate) mod font;

mod filebox;
pub use filebox::*;

mod monitor;
pub use monitor::*;

mod msgbox;
pub use msgbox::*;

mod window;
pub use window::*;

mod button;
pub use button::*;

mod canvas;
pub use canvas::*;

mod edit;
pub use edit::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;

mod accent;
pub use accent::*;

pub fn color_theme() -> ColorTheme {
    unsafe {
        if darkmode::is_dark_mode_allowed_for_app() {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    }
}

fn fix_crlf(s: &str) -> String {
    let mut v = Vec::with_capacity(s.len());
    let mut prev = 0u8;
    for &b in s.as_bytes() {
        if b == b'\n' && prev != b'\r' {
            v.push(b'\r');
        }
        v.push(b);
        prev = b;
    }
    // Safety: only ASCII operations
    unsafe { String::from_utf8_unchecked(v) }
}

#[inline]
fn with_u16c<T>(s: &str, f: impl FnOnce(&U16CStr) -> T) -> T {
    if s.len() < 32 {
        // A UTF-8 string with length < 32 is guaranteed to fit in a
        // `ArrayVec<u16, 32>`.
        let buf = s
            .encode_utf16()
            .chain([0])
            .collect::<compio::arrayvec::ArrayVec<u16, 32>>();
        f(U16CStr::from_slice_truncate(&buf).unwrap())
    } else {
        let buf = s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        f(U16CStr::from_slice_truncate(&buf).unwrap())
    }
}

// Safety: `f` must fill the buffer with null-terminated UTF-16 data.
#[inline]
unsafe fn get_u16c(len: usize, f: impl FnOnce(&mut [MaybeUninit<u16>]) -> usize) -> U16CString {
    if len == 0 {
        return U16CString::new();
    }
    let mut buf = Vec::with_capacity(len + 1);
    let len = f(buf.spare_capacity_mut());
    debug_assert!(len < buf.capacity());
    unsafe {
        buf.set_len(len + 1);
        U16CString::from_vec_unchecked(buf)
    }
}
