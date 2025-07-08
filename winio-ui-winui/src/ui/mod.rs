use windows::{
    Foundation::{IReference, PropertyValue},
    Graphics::{PointInt32, SizeInt32},
    core::{HSTRING, Interface, RuntimeType},
};
use winio_primitive::{Point, Size};

fn from_graphics_pointi32(p: PointInt32) -> Point {
    Point::new(p.X as _, p.Y as _)
}

fn to_graphics_pointi32(p: Point) -> PointInt32 {
    PointInt32 {
        X: p.x as _,
        Y: p.y as _,
    }
}

fn from_graphics_sizei32(s: SizeInt32) -> Size {
    Size::new(s.Width as _, s.Height as _)
}

fn to_graphics_sizei32(s: Size) -> SizeInt32 {
    SizeInt32 {
        Width: s.width as _,
        Height: s.height as _,
    }
}

fn from_foundation_size(s: windows::Foundation::Size) -> Size {
    Size::new(s.Width as _, s.Height as _)
}

trait ToIReference: RuntimeType {
    fn to_reference(&self) -> IReference<Self>;
}

impl ToIReference for HSTRING {
    fn to_reference(&self) -> IReference<Self> {
        PropertyValue::CreateString(self).unwrap().cast().unwrap()
    }
}

impl ToIReference for bool {
    fn to_reference(&self) -> IReference<Self> {
        PropertyValue::CreateBoolean(*self).unwrap().cast().unwrap()
    }
}

mod window;
pub use window::*;

mod widget;
pub(crate) use widget::*;

mod button;
pub use button::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;
