use std::ffi::CStr;

use objc2::rc::Retained;
use objc2_core_foundation::{CFArray, CFRange, CFString, CFStringBuiltInEncodings};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};
use objc2_ui_kit::{UIApplication, UIUserInterfaceStyle, UIWindowScene};
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

mod scroll_bar;
pub use scroll_bar::*;

mod scroll_view;
pub use scroll_view::*;

mod slider;
pub use slider::*;

#[cfg(feature = "media")]
mod media;
#[cfg(feature = "media")]
pub use media::*;

#[cfg(feature = "webview")]
mod webview;
#[cfg(feature = "webview")]
pub use webview::*;

mod accent;
pub use accent::*;

mod tab_view;
pub use tab_view::*;

// OK to use `keyWindow` because it is for application wide theme detection.
pub fn color_theme() -> crate::Result<ColorTheme> {
    let theme = first_ui_window_scene()?
        .map(|scene| {
            let trait_collection = scene.traitCollection();
            let style = unsafe { trait_collection.userInterfaceStyle() };
            match style {
                UIUserInterfaceStyle::Dark => ColorTheme::Dark,
                _ => ColorTheme::Light,
            }
        })
        .unwrap_or(ColorTheme::Light);
    Ok(theme)
}

pub(crate) fn first_ui_window_scene() -> crate::Result<Option<Retained<UIWindowScene>>> {
    let mtm = MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;
    crate::catch(|| {
        let app = UIApplication::sharedApplication(mtm);
        for scene in app.connectedScenes() {
            if let Ok(scene) = Retained::downcast::<UIWindowScene>(scene) {
                return Some(scene);
            }
        }
        None
    })
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
pub(crate) fn to_cgrect(rect: Rect) -> NSRect {
    NSRect::new(to_cgpoint(rect.origin), to_cgsize(rect.size))
}

#[inline]
pub(crate) fn from_cgrect(rect: NSRect) -> Rect {
    Rect::new(from_cgpoint(rect.origin), from_cgsize(rect.size))
}

#[inline]
pub(crate) fn to_cgpoint(p: Point) -> NSPoint {
    NSPoint::new(p.x, p.y)
}

#[inline]
pub(crate) fn from_cgpoint(p: NSPoint) -> Point {
    Point::new(p.x, p.y)
}

trait TollFreeBridge<T>: Sized {
    fn bridge(&self) -> &T {
        unsafe { &*(std::ptr::addr_of!(*self).cast::<T>()) }
    }
}

impl TollFreeBridge<CFString> for NSString {}

impl<T: ?Sized> TollFreeBridge<CFArray> for CFArray<T> {}

#[inline]
pub(crate) fn from_nsstring(s: &NSString) -> String {
    let s = s.bridge();
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
