use crate::{Brush, Color, DrawingFont, Pen, Point, Rect, Size, ui::sys};

/// Brush with single solid color.
#[derive(Debug, Clone)]
pub struct SolidColorBrush {
    pub(crate) color: Color,
}

impl SolidColorBrush {
    /// Create [`SolidColorBrush`] with color.
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

/// Pen with specified brush.
#[derive(Debug, Clone)]
pub struct BrushPen<B: Brush> {
    pub(crate) brush: B,
    pub(crate) width: f64,
}

impl<B: Brush> BrushPen<B> {
    /// Create [`BrushPen`] with brush and pen width.
    pub fn new(brush: B, width: f64) -> Self {
        Self { brush, width }
    }
}

/// Canvas drawing context.
pub struct DrawingContext<'a>(sys::DrawingContext<'a>);

impl<'a> DrawingContext<'a> {
    pub(crate) fn new(ctx: sys::DrawingContext<'a>) -> Self {
        Self(ctx)
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.0.draw_arc(pen, rect, start, end);
    }

    /// Fill a pie.
    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        self.0.fill_pie(brush, rect, start, end);
    }

    /// Draw an ellipse.
    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        self.0.draw_ellipse(pen, rect);
    }

    /// Fill an ellipse.
    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        self.0.fill_ellipse(brush, rect);
    }

    /// Draw a line.
    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        self.0.draw_line(pen, start, end);
    }

    /// Draw a rectangle.
    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        self.0.draw_rect(pen, rect);
    }

    /// Fill a rectangle.
    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        self.0.fill_rect(brush, rect);
    }

    /// Draw a rounded rectangle.
    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        self.0.draw_round_rect(pen, rect, round);
    }

    /// Fill a rounded rectangle.
    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        self.0.fill_round_rect(brush, rect, round);
    }

    /// Draw a string.
    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) {
        self.0.draw_str(brush, font, pos, text.as_ref());
    }
}
