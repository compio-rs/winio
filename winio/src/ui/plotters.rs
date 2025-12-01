use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
    FontStyle, FontTransform,
    text_anchor::{HPos, VPos},
};
use winio_primitive::{
    Angle, BrushPen, Color, DrawingFont, DrawingFontBuilder, HAlign, Layoutable, Point, Rect,
    RectBox, Rotation, Size, SolidColorBrush, Transform, VAlign,
};

use crate::{Error, ui::DrawingContext, widgets::Canvas};

/// The Plotters backend adapter for [`Canvas`].
pub struct WinioCanvasBackend<'a> {
    canvas: &'a mut Canvas,
    size: (u32, u32),
    inner: Option<DrawingContext<'a>>,
}

impl<'a> WinioCanvasBackend<'a> {
    /// Create a new [`WinioCanvasBackend`] from a [`Canvas`].
    pub fn new(canvas: &'a mut Canvas) -> Result<Self, Error> {
        let size = canvas.size()?;
        Ok(Self {
            canvas,
            size: (size.width as u32, size.height as u32),
            inner: None,
        })
    }

    fn context(&self) -> &DrawingContext<'a> {
        self.inner.as_ref().expect("Context is not prepared")
    }

    fn context_mut(&mut self) -> &mut DrawingContext<'a> {
        self.inner.as_mut().expect("Context is not prepared")
    }
}

fn bcolor(color: BackendColor) -> Color {
    Color::new(
        color.rgb.0,
        color.rgb.1,
        color.rgb.2,
        (color.alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn bpoint(p: BackendCoord) -> Point {
    Point::new(p.0 as f64, p.1 as f64)
}

fn bbrush(color: BackendColor) -> SolidColorBrush {
    SolidColorBrush::new(bcolor(color))
}

fn bpen(style: &impl BackendStyle) -> BrushPen<SolidColorBrush> {
    BrushPen::new(bbrush(style.color()), style.stroke_width() as f64)
}

fn bfont(style: &impl BackendTextStyle) -> DrawingFont {
    let mut builder = DrawingFontBuilder::new();
    builder.size(style.size());
    match style.style() {
        FontStyle::Normal => &mut builder,
        FontStyle::Bold => builder.bold(true),
        FontStyle::Italic => builder.italic(true),
        FontStyle::Oblique => builder.italic(true),
    };
    let anchor = style.anchor();
    let halign = match anchor.h_pos {
        HPos::Left => HAlign::Left,
        HPos::Center => HAlign::Center,
        HPos::Right => HAlign::Right,
    };
    builder.halign(halign);
    let valign = match anchor.v_pos {
        VPos::Top => VAlign::Top,
        VPos::Center => VAlign::Center,
        VPos::Bottom => VAlign::Bottom,
    };
    builder.valign(valign);
    builder.build()
}

impl<'a> DrawingBackend for WinioCanvasBackend<'a> {
    type ErrorType = Error;

    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if self.inner.is_none() {
            // Update size before creating context.
            let size = self.canvas.size().map_err(DrawingErrorKind::DrawingError)?;
            self.size = (size.width as u32, size.height as u32);
            // SAFETY: we don't expose the context out of this struct,
            // and don't use the canvas while the context is alive.
            self.inner = Some(unsafe {
                std::mem::transmute(
                    self.canvas
                        .context()
                        .map_err(DrawingErrorKind::DrawingError)?,
                )
            });
        }
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if let Some(context) = self.inner.take() {
            context.close().map_err(DrawingErrorKind::DrawingError)?;
        }
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.context_mut()
            .fill_rect(bbrush(color), Rect::new(bpoint(point), Size::new(1.0, 1.0)))
            .map_err(DrawingErrorKind::DrawingError)
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.context_mut()
            .draw_line(bpen(style), bpoint(from), bpoint(to))
            .map_err(DrawingErrorKind::DrawingError)
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let rect = RectBox::new(bpoint(upper_left), bpoint(bottom_right)).to_rect();
        let context = self.context_mut();
        if fill {
            context.fill_rect(bbrush(style.color()), rect)
        } else {
            context.draw_rect(bpen(style), rect)
        }
        .map_err(DrawingErrorKind::DrawingError)
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let context = self.context_mut();
        let mut iter = path.into_iter();
        let Some(start) = iter.next() else {
            return Ok(());
        };
        (|| {
            let mut builder = context.create_path_builder(bpoint(start))?;
            for p in iter {
                builder.add_line(bpoint(p))?;
            }
            let path = builder.build(false)?;
            context.draw_path(bpen(style), &path)
        })()
        .map_err(DrawingErrorKind::DrawingError)
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let context = self.context_mut();
        let rect = Rect::new(
            Point::new(
                center.0 as f64 - radius as f64,
                center.1 as f64 - radius as f64,
            ),
            Size::new((radius * 2) as f64, (radius * 2) as f64),
        );
        if fill {
            context.fill_ellipse(bbrush(style.color()), rect)
        } else {
            context.draw_ellipse(bpen(style), rect)
        }
        .map_err(DrawingErrorKind::DrawingError)
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let context = self.context_mut();
        let mut iter = vert.into_iter();
        let Some(start) = iter.next() else {
            return Ok(());
        };
        (|| {
            let mut min_x = start.0;
            let mut max_x = start.0;
            let mut min_y = start.1;
            let mut max_y = start.1;
            let mut builder = context.create_path_builder(bpoint(start))?;
            for p in iter {
                min_x = min_x.min(p.0);
                max_x = max_x.max(p.0);
                min_y = min_y.min(p.1);
                max_y = max_y.max(p.1);
                builder.add_line(bpoint(p))?;
            }
            let path = builder.build(true)?;
            if min_x == max_x || min_y == max_y {
                context.draw_line(bpen(style), bpoint((min_x, min_y)), bpoint((max_x, max_y)))
            } else {
                context.fill_path(bbrush(style.color()), &path)
            }
        })()
        .map_err(DrawingErrorKind::DrawingError)
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let context = self.context_mut();
        let font = bfont(style);
        let pos = bpoint(pos);
        let transform = style.transform();
        let rotate = match transform {
            FontTransform::None => Rotation::identity(),
            FontTransform::Rotate90 => Rotation::new(Angle::degrees(90.0)),
            FontTransform::Rotate180 => Rotation::new(Angle::degrees(180.0)),
            FontTransform::Rotate270 => Rotation::new(Angle::degrees(270.0)),
        };
        let need_transform = rotate != Rotation::identity();
        (|| {
            if need_transform {
                context.set_transform(
                    rotate
                        .to_transform()
                        .pre_translate(-pos.to_vector())
                        .then_translate(pos.to_vector()),
                )?;
            }
            context.draw_str(bbrush(style.color()), font, pos, text)?;
            if need_transform {
                context.set_transform(Transform::identity())?;
            }
            Ok(())
        })()
        .map_err(DrawingErrorKind::DrawingError)
    }

    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        let font = bfont(style);
        let size = self
            .context()
            .measure_str(font, text)
            .map_err(DrawingErrorKind::DrawingError)?;
        let (width, height) = style
            .transform()
            .transform(size.width as _, size.height as _);
        Ok((width as u32, height as u32))
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let image = image::DynamicImage::ImageRgb8(
            image::ImageBuffer::from_vec(iw, ih, src.to_vec()).expect("Invalid image buffer"),
        );
        let context = self.context_mut();
        (|| {
            let drawing_image = context.create_image(image)?;
            context.draw_image(
                &drawing_image,
                Rect::new(bpoint(pos), Size::new(iw as f64, ih as f64)),
                None,
            )
        })()
        .map_err(DrawingErrorKind::DrawingError)
    }
}
