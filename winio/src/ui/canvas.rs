use image::DynamicImage;
use winio_primitive::{DrawingFont, Point, Rect, Size};

use crate::{sys, sys::Result};

/// Drawing brush.
pub trait Brush: sys::Brush {}

impl<B: sys::Brush> Brush for B {}

/// Drawing Pen.
pub trait Pen: sys::Pen {}

impl<P: sys::Pen> Pen for P {}

/// Canvas compatible drawing image.
pub struct DrawingImage(sys::DrawingImage);

impl DrawingImage {
    /// Size of the image.
    pub fn size(&self) -> Result<Size> {
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
    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) -> Result<()> {
        self.0.draw_path(pen, &path.0)
    }

    /// Fill a path.
    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) -> Result<()> {
        self.0.fill_path(brush, &path.0)
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        self.0.draw_arc(pen, fix_rect(rect), start, end)
    }

    /// Draw an arc.
    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        self.0.draw_pie(pen, fix_rect(rect), start, end)
    }

    /// Fill a pie.
    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> Result<()> {
        self.0.fill_pie(brush, fix_rect(rect), start, end)
    }

    /// Draw an ellipse.
    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        self.0.draw_ellipse(pen, fix_rect(rect))
    }

    /// Fill an ellipse.
    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        self.0.fill_ellipse(brush, fix_rect(rect))
    }

    /// Draw a line.
    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) -> Result<()> {
        self.0.draw_line(pen, start, end)
    }

    /// Draw a rectangle.
    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        self.0.draw_rect(pen, fix_rect(rect))
    }

    /// Fill a rectangle.
    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        self.0.fill_rect(brush, fix_rect(rect))
    }

    /// Draw a rounded rectangle.
    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) -> Result<()> {
        self.0.draw_round_rect(pen, fix_rect(rect), round)
    }

    /// Fill a rounded rectangle.
    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) -> Result<()> {
        self.0.fill_round_rect(brush, fix_rect(rect), round)
    }

    /// Draw a string.
    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) -> Result<()> {
        self.0.draw_str(brush, fix_font(font), pos, text.as_ref())
    }

    /// Create a [`DrawingContext`]-compatible image from [`DynamicImage`].
    pub fn create_image(&self, image: DynamicImage) -> Result<DrawingImage> {
        Ok(DrawingImage(self.0.create_image(image)?))
    }

    /// Draw a image with RGBA format.
    pub fn draw_image(
        &mut self,
        image: &DrawingImage,
        rect: Rect,
        clip: Option<Rect>,
    ) -> Result<()> {
        self.0.draw_image(&image.0, fix_rect(rect), clip)
    }

    /// Create [`DrawingPathBuilder`].
    pub fn create_path_builder(&self, start: Point) -> Result<DrawingPathBuilder> {
        Ok(DrawingPathBuilder(self.0.create_path_builder(start)?))
    }
}

/// A drawing path.
pub struct DrawingPath(sys::DrawingPath);

/// Builder for [`DrawingPath`].
pub struct DrawingPathBuilder(sys::DrawingPathBuilder);

impl DrawingPathBuilder {
    /// Line from current point to the target point.
    pub fn add_line(&mut self, p: Point) -> Result<()> {
        self.0.add_line(p)?;
        Ok(())
    }

    /// Add arc. A line will be created implicitly if the start point is not the
    /// current point.
    pub fn add_arc(
        &mut self,
        center: Point,
        radius: Size,
        start: f64,
        end: f64,
        clockwise: bool,
    ) -> Result<()> {
        self.0
            .add_arc(center, fix_size(radius), start, end, clockwise)?;
        Ok(())
    }

    /// Add a cubic Bezier curve.
    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) -> Result<()> {
        self.0.add_bezier(p1, p2, p3)?;
        Ok(())
    }

    /// Build [`DrawingPath`].
    pub fn build(self, close: bool) -> Result<DrawingPath> {
        Ok(DrawingPath(self.0.build(close)?))
    }
}
