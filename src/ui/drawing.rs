use euclid::*;
use rgb::RGBA8;

/// The logical space.
pub struct LogicalSpace;

/// Logical point.
pub type Point = Point2D<f64, LogicalSpace>;
/// Logical vector.
pub type Vector = Vector2D<f64, LogicalSpace>;
/// Logical size.
pub type Size = Size2D<f64, LogicalSpace>;
/// Logical rectangle.
pub type Rect = euclid::Rect<f64, LogicalSpace>;
/// Logical rectangle box.
pub type RectBox = Box2D<f64, LogicalSpace>;
/// Logical margin.
pub type Margin = SideOffsets2D<f64, LogicalSpace>;
/// Logical rotation.
pub type Rotation = Rotation2D<f64, LogicalSpace, LogicalSpace>;
/// Angle of vector.
pub type Angle = euclid::Angle<f64>;

/// The relative space, which maps [0.0, 1.0] to the logical space.
pub struct RelativeSpace;

/// Relative point.
pub type RelativePoint = Point2D<f64, RelativeSpace>;
/// Relative vector.
pub type RelativeVector = Vector2D<f64, RelativeSpace>;
/// Relative size.
pub type RelativeSize = Size2D<f64, RelativeSpace>;

/// Transform from the relative space to the logical space.
pub type RelativeToLogical = Transform2D<f64, RelativeSpace, LogicalSpace>;

/// Color theme of application.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum ColorTheme {
    /// Default light theme.
    Light,
    /// Dark theme.
    Dark,
}

impl ColorTheme {
    /// Get the current color theme.
    pub fn current() -> Self {
        crate::ui::color_theme()
    }
}

/// Horizontal alignment.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HAlign {
    /// Left aligned.
    Left,
    /// Horizontal centered.
    Center,
    /// Right aligned.
    Right,
}

/// Vertical alignment.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VAlign {
    /// Top aligned.
    Top,
    /// Vertical centered.
    Center,
    /// Bottom aligned.
    Bottom,
}

/// Color type.
pub type Color = RGBA8;

/// Font for drawing.
#[derive(Debug, Clone)]
pub struct DrawingFont {
    /// Font name.
    pub family: String,
    /// Font size.
    pub size: f64,
    /// *Italic*.
    pub italic: bool,
    /// **Bold**.
    pub bold: bool,
    /// Horizontal alignment.
    pub halign: HAlign,
    /// Vertical alignment.
    pub valign: VAlign,
}

impl DrawingFont {
    /// Create a builder.
    pub fn builder() -> DrawingFontBuilder {
        DrawingFontBuilder::new()
    }
}

/// Builder of [`DrawingFont`].
pub struct DrawingFontBuilder {
    value: DrawingFont,
}

impl Default for DrawingFontBuilder {
    fn default() -> Self {
        Self {
            value: DrawingFont {
                family: String::new(),
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
    /// Create a builder for [`DrawingFont`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Font name.
    pub fn family(&mut self, s: impl AsRef<str>) -> &mut Self {
        self.value.family = s.as_ref().to_string();
        self
    }

    /// Font size.
    pub fn size(&mut self, s: f64) -> &mut Self {
        self.value.size = s;
        self
    }

    /// *Italic*.
    pub fn italic(&mut self, v: bool) -> &mut Self {
        self.value.italic = v;
        self
    }

    /// **Bold**.
    pub fn bold(&mut self, v: bool) -> &mut Self {
        self.value.bold = v;
        self
    }

    /// Horizontal alignment.
    pub fn halign(&mut self, v: HAlign) -> &mut Self {
        self.value.halign = v;
        self
    }

    /// Vertical alignment.
    pub fn valign(&mut self, v: VAlign) -> &mut Self {
        self.value.valign = v;
        self
    }

    /// Build [`DrawingFont`].
    pub fn build(&self) -> DrawingFont {
        self.value.clone()
    }
}

/// Represents the mouse button.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle button.
    Middle,
    /// Other buttons.
    Other,
}
