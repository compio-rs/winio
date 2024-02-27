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

mod callback;

use icrate::Foundation::{CGPoint, CGRect, CGSize, NSString};

use crate::{Point, Rect, Size};

#[inline]
fn from_cgsize(size: CGSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
fn from_cgpoint(p: CGPoint) -> Point {
    Point::new(p.x, p.y)
}

#[inline]
fn from_cgrect(rect: CGRect) -> Rect {
    Rect::new(from_cgpoint(rect.origin), from_cgsize(rect.size))
}

#[inline]
fn to_cgsize(size: Size) -> CGSize {
    CGSize::new(size.width, size.height)
}

#[inline]
fn to_cgpoint(p: Point) -> CGPoint {
    CGPoint::new(p.x, p.y)
}

#[inline]
fn to_cgrect(rect: Rect) -> CGRect {
    CGRect::new(to_cgpoint(rect.origin), to_cgsize(rect.size))
}

fn from_nsstring(s: &NSString) -> String {
    String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(s.UTF8String()) }.to_bytes())
        .into_owned()
}
