mod window;
use icrate::Foundation::{CGPoint, CGRect, CGSize};
pub use window::*;

use crate::{Point, Rect, Size};

mod callback;

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
