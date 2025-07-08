use windows::{
    Foundation::{IReference, PropertyValue},
    Graphics::{PointInt32, SizeInt32},
    core::{HSTRING, Interface},
};
use winio_primitive::{Point, Size};

pub(crate) fn from_graphics_pointi32(p: PointInt32) -> Point {
    Point::new(p.X as _, p.Y as _)
}

pub(crate) fn to_graphics_pointi32(p: Point) -> PointInt32 {
    PointInt32 {
        X: p.x as _,
        Y: p.y as _,
    }
}

pub(crate) fn from_graphics_sizei32(s: SizeInt32) -> Size {
    Size::new(s.Width as _, s.Height as _)
}

pub(crate) fn to_graphics_sizei32(s: Size) -> SizeInt32 {
    SizeInt32 {
        Width: s.width as _,
        Height: s.height as _,
    }
}

pub(crate) fn from_foundation_size(s: windows::Foundation::Size) -> Size {
    Size::new(s.Width as _, s.Height as _)
}

pub(crate) fn hstring_ref(text: &HSTRING) -> IReference<HSTRING> {
    PropertyValue::CreateString(text).unwrap().cast().unwrap()
}

mod window;
pub use window::*;

mod widget;
pub(crate) use widget::*;

mod button;
pub use button::*;
