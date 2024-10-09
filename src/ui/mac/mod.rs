mod canvas;
pub use canvas::*;

mod window;
pub use window::*;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

/// [`NSWindow`].
pub type RawWindow = Id<NSWindow>;

use objc2::rc::Id;
use objc2_app_kit::NSWindow;
use objc2_foundation::{CGSize, NSString, NSUserDefaults};

use crate::{ColorTheme, Size};

pub fn color_theme() -> ColorTheme {
    unsafe {
        let osx_mode = NSUserDefaults::standardUserDefaults()
            .stringForKey(&NSString::from_str("AppleInterfaceStyle"));
        let is_dark = osx_mode
            .map(|mode| mode.isEqualToString(&NSString::from_str("Dark")))
            .unwrap_or_default();
        if is_dark {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    }
}

#[inline]
pub(crate) fn from_cgsize(size: CGSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
pub(crate) fn to_cgsize(size: Size) -> CGSize {
    CGSize::new(size.width, size.height)
}

pub(crate) fn from_nsstring(s: &NSString) -> String {
    String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(s.UTF8String()) }.to_bytes())
        .into_owned()
}
