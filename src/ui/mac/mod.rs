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
use objc2_foundation::{CGSize, NSString};

use crate::Size;

#[inline]
fn from_cgsize(size: CGSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
fn to_cgsize(size: Size) -> CGSize {
    CGSize::new(size.width, size.height)
}

fn from_nsstring(s: &NSString) -> String {
    String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(s.UTF8String()) }.to_bytes())
        .into_owned()
}
