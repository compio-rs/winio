use euclid::*;
use rgb::RGBA8;
use widestring::U16CString;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Orient {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HAlign {
    Left,
    Center,
    Right,
    Stretch,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
    Stretch,
}

pub type Color = RGBA8;

#[derive(Debug, Clone)]
pub struct DrawingFont {
    pub family: U16CString,
    pub size: f64,
    pub italic: bool,
    pub bold: bool,
    pub halign: HAlign,
    pub valign: VAlign,
}

pub struct DrawingFontBuilder {
    value: DrawingFont,
}

impl Default for DrawingFontBuilder {
    fn default() -> Self {
        Self {
            value: DrawingFont {
                family: U16CString::default(),
                size: 0.0,
                italic: false,
                bold: false,
                halign: HAlign::Left,
                valign: VAlign::Top,
            },
        }
    }
}

impl DrawingFontBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn family(&mut self, s: impl AsRef<str>) -> &mut Self {
        self.value.family = U16CString::from_str_truncate(s.as_ref());
        self
    }

    pub fn size(&mut self, s: f64) -> &mut Self {
        self.value.size = s;
        self
    }

    pub fn italic(&mut self, v: bool) -> &mut Self {
        self.value.italic = v;
        self
    }

    pub fn bold(&mut self, v: bool) -> &mut Self {
        self.value.bold = v;
        self
    }

    pub fn halign(&mut self, v: HAlign) -> &mut Self {
        self.value.halign = v;
        self
    }

    pub fn valign(&mut self, v: VAlign) -> &mut Self {
        self.value.valign = v;
        self
    }

    pub fn build(&self) -> DrawingFont {
        self.value.clone()
    }
}
