use image::DynamicImage;

use crate::{
    Brush, Color, DrawingFont, Pen, Point, Rect, RelativePoint, RelativeSize, Size, ui::sys,
};

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

/// A transition point in a gradient.
#[derive(Debug, PartialEq, Clone)]
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
pub struct LinearGradientBrush {
    pub(crate) stops: Vec<GradientStop>,
    pub(crate) start: RelativePoint,
    pub(crate) end: RelativePoint,
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
pub struct RadialGradientBrush {
    pub(crate) stops: Vec<GradientStop>,
    pub(crate) origin: RelativePoint,
    pub(crate) center: RelativePoint,
    pub(crate) radius: RelativeSize,
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

/// Canvas compatible drawing image.
pub struct DrawingImage(sys::DrawingImage);

impl DrawingImage {
    /// Size of the image.
    pub fn size(&self) -> Size {
        self.0.size()
    }
}

/// Canvas drawing context.
pub struct DrawingContext<'a>(sys::DrawingContext<'a>);

#[inline]
fn fix_rect(mut rect: Rect) -> Rect {
    rect.size = fix_size(rect.size);
    rect
}

#[inline]
fn fix_size(mut size: Size) -> Size {
    size.width = size.width.max(0.1);
    size.height = size.height.max(0.1);
    size
}

#[inline]
fn fix_font(mut font: DrawingFont) -> DrawingFont {
    font.size = font.size.max(0.1);
    font
}

impl<'a> DrawingContext<'a> {
    pub(crate) fn new(ctx: sys::DrawingContext<'a>) -> Self {
        Self(ctx)
    }

    /// Draw a path.
    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) {
        self.0.draw_path(pen, &path.0);
    }

    /// Fill a path.
    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) {
        self.0.fill_path(brush, &path.0);
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.0.draw_arc(pen, fix_rect(rect), start, end);
    }

    /// Draw an arc.
    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.0.draw_pie(pen, fix_rect(rect), start, end);
    }

    /// Fill a pie.
    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        self.0.fill_pie(brush, fix_rect(rect), start, end);
    }

    /// Draw an ellipse.
    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        self.0.draw_ellipse(pen, fix_rect(rect));
    }

    /// Fill an ellipse.
    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        self.0.fill_ellipse(brush, fix_rect(rect));
    }

    /// Draw a line.
    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        self.0.draw_line(pen, start, end);
    }

    /// Draw a rectangle.
    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        self.0.draw_rect(pen, fix_rect(rect));
    }

    /// Fill a rectangle.
    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        self.0.fill_rect(brush, fix_rect(rect));
    }

    /// Draw a rounded rectangle.
    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        self.0.draw_round_rect(pen, fix_rect(rect), round);
    }

    /// Fill a rounded rectangle.
    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        self.0.fill_round_rect(brush, fix_rect(rect), round);
    }

    /// Draw a string.
    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) {
        self.0.draw_str(brush, fix_font(font), pos, text.as_ref());
    }

    /// Create a [`DrawingContext`]-compatible image from [`DynamicImage`].
    pub fn create_image(&self, image: DynamicImage) -> DrawingImage {
        DrawingImage(self.0.create_image(image))
    }

    /// Draw a image with RGBA format.
    pub fn draw_image(&mut self, image: &DrawingImage, rect: Rect, clip: Option<Rect>) {
        self.0.draw_image(&image.0, fix_rect(rect), clip);
    }

    /// Create [`DrawingPathBuilder`].
    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder {
        DrawingPathBuilder(self.0.create_path_builder(start))
    }
}

/// A drawing path.
pub struct DrawingPath(sys::DrawingPath);

/// Builder for [`DrawingPath`].
pub struct DrawingPathBuilder(sys::DrawingPathBuilder);

impl DrawingPathBuilder {
    /// Line from current point to the target point.
    pub fn add_line(&mut self, p: Point) {
        self.0.add_line(p);
    }

    /// Add arc. A line will be created implicitly if the start point is not the
    /// current point.
    pub fn add_arc(&mut self, center: Point, radius: Size, start: f64, end: f64, clockwise: bool) {
        self.0
            .add_arc(center, fix_size(radius), start, end, clockwise);
    }

    /// Add a cubic Bezier curve.
    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) {
        self.0.add_bezier(p1, p2, p3);
    }

    /// Build [`DrawingPath`].
    pub fn build(self, close: bool) -> DrawingPath {
        DrawingPath(self.0.build(close))
    }
}
