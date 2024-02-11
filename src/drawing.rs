use euclid::*;
use rgb::RGBA8;

pub struct ScreenSpace;

pub type Point = Point2D<f64, ScreenSpace>;
pub type Vector = Vector2D<f64, ScreenSpace>;
pub type Size = Size2D<f64, ScreenSpace>;
pub type Rect = euclid::Rect<f64, ScreenSpace>;
pub type RectBox = Box2D<f64, ScreenSpace>;
pub type Margin = SideOffsets2D<f64, ScreenSpace>;
pub type Rotation = Rotation2D<f64, ScreenSpace, ScreenSpace>;

pub struct RelativeSpace;

pub type RelativePoint = Point2D<f64, RelativeSpace>;
pub type RelativeVector = Vector2D<f64, RelativeSpace>;
pub type RelativeSize = Size2D<f64, RelativeSpace>;

pub type RelativeToScreen = Transform2D<f64, RelativeSpace, ScreenSpace>;

pub type Color = RGBA8;
