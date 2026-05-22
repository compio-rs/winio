use objc2_foundation::{NSPoint, NSRect, NSSize};
use winio_primitive::{Point, Rect, Size};

#[inline]
pub fn from_cgsize(size: NSSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
pub fn to_cgsize(size: Size) -> NSSize {
    NSSize::new(size.width, size.height)
}

#[inline]
pub fn to_cgrect(rect: Rect) -> NSRect {
    NSRect::new(to_cgpoint(rect.origin), to_cgsize(rect.size))
}

#[inline]
pub fn from_cgrect(rect: NSRect) -> Rect {
    Rect::new(from_cgpoint(rect.origin), from_cgsize(rect.size))
}

#[inline]
pub fn to_cgpoint(p: Point) -> NSPoint {
    NSPoint::new(p.x, p.y)
}

#[inline]
pub fn from_cgpoint(p: NSPoint) -> Point {
    Point::new(p.x, p.y)
}

#[inline]
pub fn transform_rect(s: Size, rect: Rect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    )
}

#[inline]
pub fn transform_cgrect(s: Size, rect: NSRect) -> Rect {
    Rect::new(
        Point::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        from_cgsize(rect.size),
    )
}

#[inline]
pub fn transform_point(s: Size, p: Point) -> NSPoint {
    NSPoint::new(p.x, s.height - p.y)
}

#[inline]
pub fn transform_cgpoint(s: Size, p: NSPoint) -> Point {
    Point::new(p.x, s.height - p.y)
}
