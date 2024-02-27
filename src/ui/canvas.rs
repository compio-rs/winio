use crate::{Brush, Color};

#[derive(Debug, Clone)]
pub struct SolidColorBrush {
    color: Color,
}

impl SolidColorBrush {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone)]
pub struct BrushPen<B: Brush> {
    brush: B,
    width: f64,
}

impl<B: Brush> BrushPen<B> {
    pub fn new(brush: B, width: f64) -> Self {
        Self { brush, width }
    }
}
