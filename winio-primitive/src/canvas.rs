use crate::{Color, RelativePoint, RelativeSize};

/// Brush with single solid color.
#[derive(Debug, Clone)]
pub struct SolidColorBrush {
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
    pub stops: Vec<GradientStop>,
    pub start: RelativePoint,
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
    pub stops: Vec<GradientStop>,
    pub origin: RelativePoint,
    pub center: RelativePoint,
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
    pub brush: B,
    pub width: f64,
}

impl<B> BrushPen<B> {
    /// Create [`BrushPen`] with brush and pen width.
    pub fn new(brush: B, width: f64) -> Self {
        Self { brush, width }
    }
}
