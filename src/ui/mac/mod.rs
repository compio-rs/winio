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

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

/// [`NSWindow`].
pub type RawWindow = Id<NSWindow>;

use objc2::rc::{Id, autoreleasepool};
use objc2_app_kit::NSWindow;
use objc2_foundation::{CGPoint, CGRect, CGSize, NSString, NSUserDefaults, ns_string};

use crate::{ColorTheme, Point, Rect, Size};

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
pub(crate) fn from_cgsize(size: CGSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
pub(crate) fn to_cgsize(size: Size) -> CGSize {
    CGSize::new(size.width, size.height)
}

#[inline]
pub(crate) fn transform_rect(s: Size, rect: Rect) -> CGRect {
    CGRect::new(
        CGPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    )
}

#[inline]
pub(crate) fn transform_cgrect(s: Size, rect: CGRect) -> Rect {
    Rect::new(
        Point::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        from_cgsize(rect.size),
    )
}

#[inline]
pub(crate) fn transform_point(s: Size, p: Point) -> CGPoint {
    CGPoint::new(p.x, s.height - p.y)
}

#[inline]
pub(crate) fn transform_cgpoint(s: Size, p: CGPoint) -> Point {
    Point::new(p.x, s.height - p.y)
}

#[inline]
pub(crate) fn from_nsstring(s: &NSString) -> String {
    autoreleasepool(|pool| s.as_str(pool).to_string())
}
