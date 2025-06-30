use crate::{Color, RelativePoint, RelativeSize};

/// Brush with single solid color.
#[derive(Debug, Clone)]
pub struct SolidColorBrush {
    /// The color of the brush.
    pub color: Color,
}

impl SolidColorBrush {
    /// Create [`SolidColorBrush`] with color.
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

/// A transition point in a gradient.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GradientStop {
    /// Color of the stop.
    pub color: Color,
    /// Relative position of the stop, from 0 to 1.
    pub pos: f64,
}

impl GradientStop {
    /// Create [`GradientStop`].
    pub fn new(color: Color, pos: f64) -> Self {
        Self { color, pos }
    }
}

impl From<(Color, f64)> for GradientStop {
    fn from((color, pos): (Color, f64)) -> Self {
        Self::new(color, pos)
    }
}

/// Linear gradient brush.
#[derive(Debug, Clone)]
pub struct LinearGradientBrush {
    /// The gradient stops.
    pub stops: Vec<GradientStop>,
    /// The relative start position.
    pub start: RelativePoint,
    /// The relative end position.
    pub end: RelativePoint,
}

impl LinearGradientBrush {
    /// Create [`LinearGradientBrush`].
    pub fn new(
        stops: impl IntoIterator<Item = GradientStop>,
        start: RelativePoint,
        end: RelativePoint,
    ) -> Self {
        Self {
            stops: stops.into_iter().collect(),
            start,
            end,
        }
    }
}

/// Radial gradient brush.
#[derive(Debug, Clone)]
pub struct RadialGradientBrush {
    /// The gradient stops.
    pub stops: Vec<GradientStop>,
    /// The relative origin position.
    pub origin: RelativePoint,
    /// The relative center position.
    pub center: RelativePoint,
    /// The relative radius.
    pub radius: RelativeSize,
}

impl RadialGradientBrush {
    /// Create [`RadialGradientBrush`].
    pub fn new(
        stops: impl IntoIterator<Item = GradientStop>,
        origin: RelativePoint,
        center: RelativePoint,
        radius: RelativeSize,
    ) -> Self {
        Self {
            stops: stops.into_iter().collect(),
            origin,
            center,
            radius,
        }
    }
}

/// Pen with specified brush.
#[derive(Debug, Clone)]
pub struct BrushPen<B> {
    /// The inner brush.
    pub brush: B,
    /// The width of the pen.
    pub width: f64,
}

impl<B> BrushPen<B> {
    /// Create [`BrushPen`] with brush and pen width.
    pub fn new(brush: B, width: f64) -> Self {
        Self { brush, width }
    }
}
