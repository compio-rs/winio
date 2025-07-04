use std::ffi::CStr;

use objc2_core_foundation::{CFRange, CFString, CFStringBuiltInEncodings};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, NSUserDefaults, ns_string};
use winio_primitive::{ColorTheme, Point, Rect, Size};

mod canvas;
pub use canvas::*;

mod window;
pub use window::*;

mod monitor;
pub use monitor::*;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod accent;
pub use accent::*;

pub fn color_theme() -> ColorTheme {
    unsafe {
        let osx_mode =
            NSUserDefaults::standardUserDefaults().stringForKey(ns_string!("AppleInterfaceStyle"));
        let is_dark = osx_mode
            .map(|mode| mode.isEqualToString(ns_string!("Dark")))
            .unwrap_or_default();
        if is_dark {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    }
}

#[inline]
pub(crate) fn from_cgsize(size: NSSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
pub(crate) fn to_cgsize(size: Size) -> NSSize {
    NSSize::new(size.width, size.height)
}

#[inline]
pub(crate) fn transform_rect(s: Size, rect: Rect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    )
}

#[inline]
pub(crate) fn transform_cgrect(s: Size, rect: NSRect) -> Rect {
    Rect::new(
        Point::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        from_cgsize(rect.size),
    )
}

#[inline]
pub(crate) fn transform_point(s: Size, p: Point) -> NSPoint {
    NSPoint::new(p.x, s.height - p.y)
}

#[inline]
pub(crate) fn transform_cgpoint(s: Size, p: NSPoint) -> Point {
    Point::new(p.x, s.height - p.y)
}

#[inline]
pub(crate) fn from_nsstring(s: &NSString) -> String {
    let s = unsafe { &*(std::ptr::addr_of!(*s).cast::<CFString>()) };
    // UTF16 length
    let len = s.length() as usize;
    if len == 0 {
        return String::new();
    }

    let mut ptr = s.c_string_ptr(CFStringBuiltInEncodings::EncodingUTF8.0);
    if ptr.is_null() {
        ptr = s.c_string_ptr(CFStringBuiltInEncodings::EncodingASCII.0);
    }
    if !ptr.is_null() {
        unsafe {
            let str = CStr::from_ptr(ptr);
            String::from_utf8_unchecked(str.to_bytes().to_vec())
        }
    } else {
        let ptr = s.characters_ptr();
        if !ptr.is_null() {
            String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
        } else {
            let mut buffer = Vec::<u16>::with_capacity(len);
            unsafe {
                s.characters(CFRange::new(0, len as isize), buffer.as_mut_ptr());
                buffer.set_len(len);
            }
            String::from_utf16_lossy(&buffer)
        }
    }
}
