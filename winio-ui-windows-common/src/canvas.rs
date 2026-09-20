use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    mem::MaybeUninit,
};

use compio_log::error;
use image::{DynamicImage, RgbaImage};
use widestring::U16CString;
use windows::Win32::Graphics::{
    Direct2D::{
        Common::{
            D2D_MATRIX_3X2_F, D2D_MATRIX_3X2_F_0, D2D_MATRIX_3X2_F_0_1, D2D_POINT_2F, D2D_RECT_F,
            D2D_SIZE_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BEZIER_SEGMENT, D2D1_COLOR_F,
            D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN,
            D2D1_GRADIENT_STOP, D2D1_PIXEL_FORMAT,
        },
        D2D1_ARC_SEGMENT, D2D1_ARC_SIZE_LARGE, D2D1_ARC_SIZE_SMALL,
        D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR, D2D1_BITMAP_PROPERTIES,
        D2D1_BRUSH_PROPERTIES, D2D1_DEFAULT_FLATTENING_TOLERANCE,
        D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP,
        D2D1_FEATURE_LEVEL_DEFAULT, D2D1_GAMMA_2_2, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
        D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES,
        D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
        D2D1_SWEEP_DIRECTION_CLOCKWISE, D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE, ID2D1Bitmap,
        ID2D1Brush, ID2D1Factory, ID2D1Geometry, ID2D1GeometrySink, ID2D1PathGeometry,
        ID2D1RenderTarget,
    },
    DirectWrite::{
        DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_NORMAL, IDWriteFactory, IDWriteTextLayout,
    },
    Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    Imaging::{
        GUID_WICPixelFormat32bppPBGRA, IWICBitmap, WICBitmapCacheOnLoad, WICBitmapLockWrite,
    },
};
use windows_core::Interface;
use winio_primitive::{
    BitmapRect, BitmapSize, BrushPen, Color, Font, GradientStop, LinearGradientBrush, Point,
    RadialGradientBrush, Rect, RectBox, RelativePoint, RelativeToLogical, Size, SolidColorBrush,
    Transform, Vector,
};

use crate::{Error, Result, d2d1_factory, dwrite_factory};

fn color_f(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: c.r as f32 / 255.0,
        g: c.g as f32 / 255.0,
        b: c.b as f32 / 255.0,
        a: c.a as f32 / 255.0,
    }
}

const fn point_2f(p: Point) -> D2D_POINT_2F {
    D2D_POINT_2F {
        x: p.x as f32,
        y: p.y as f32,
    }
}

const fn size_f(s: Size) -> D2D_SIZE_F {
    D2D_SIZE_F {
        width: s.width as f32,
        height: s.height as f32,
    }
}

fn rect_f(r: Rect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: r.origin.x as f32,
        top: r.origin.y as f32,
        right: (r.origin.x + r.size.width) as f32,
        bottom: (r.origin.y + r.size.height) as f32,
    }
}

fn bitmap_rect_f(r: BitmapRect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: r.origin.x as f32,
        top: r.origin.y as f32,
        right: r.max_x() as f32,
        bottom: r.max_y() as f32,
    }
}

/// Convert straight RGBA pixels to premultiplied BGRA in place.
///
/// The length of `pixels` must be a multiple of 4.
pub fn rgba8_to_pbgra8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let a = a as u16;
        pixel[0] = ((b as u16 * a + 127) / 255) as u8;
        pixel[1] = ((g as u16 * a + 127) / 255) as u8;
        pixel[2] = ((r as u16 * a + 127) / 255) as u8;
    }
}

/// Convert premultiplied BGRA pixels to straight RGBA in place.
fn pbgra8_to_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let [b, g, r, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let alpha = a as u16;
        let (r, g, b) = if a == 0 {
            (0, 0, 0)
        } else {
            (
                ((r as u16 * 255 + alpha / 2) / alpha).min(255) as u8,
                ((g as u16 * 255 + alpha / 2) / alpha).min(255) as u8,
                ((b as u16 * 255 + alpha / 2) / alpha).min(255) as u8,
            )
        };
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
    }
}

const fn matrix_f(m: Transform) -> D2D_MATRIX_3X2_F {
    D2D_MATRIX_3X2_F {
        Anonymous: D2D_MATRIX_3X2_F_0 {
            Anonymous2: D2D_MATRIX_3X2_F_0_1 {
                _11: m.m11 as _,
                _12: m.m12 as _,
                _21: m.m21 as _,
                _22: m.m22 as _,
                _31: m.m31 as _,
                _32: m.m32 as _,
            },
        },
    }
}

fn gradient_stop(s: &GradientStop) -> D2D1_GRADIENT_STOP {
    D2D1_GRADIENT_STOP {
        position: s.pos as f32,
        color: color_f(s.color),
    }
}

/// The owner of a [`DrawingContext`], ending the drawing in a backend-specific
/// way.
pub trait ContextOwner {
    /// Finish the drawing on `target`.
    fn end_draw(&mut self, target: &ID2D1RenderTarget) -> Result<()>;
}

pub struct DrawingContext<'a> {
    d2d: ID2D1Factory,
    dwrite: IDWriteFactory,
    target: ID2D1RenderTarget,
    owner: Option<&'a mut dyn ContextOwner>,
    ended: bool,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.end_draw() {
            error!("EndDraw: {e:?}");
        }
    }
}

#[inline]
fn to_trans(rect: Rect) -> RelativeToLogical {
    RelativeToLogical::scale(rect.size.width, rect.size.height)
        .then_translate(rect.origin.to_vector())
}

fn get_arc(rect: Rect, start: f64, end: f64) -> (Size, Point, Point, Point) {
    let radius = rect.size / 2.0;
    let centerp = rect.origin.add_size(&radius);
    let startp = centerp + Vector::new(radius.width * start.cos(), radius.height * start.sin());
    let endp = centerp + Vector::new(radius.width * end.cos(), radius.height * end.sin());
    (radius, centerp, startp, endp)
}

fn ellipse(rect: Rect) -> D2D1_ELLIPSE {
    D2D1_ELLIPSE {
        point: point_2f(rect.origin.add_size(&(rect.size / 2.0))),
        radiusX: (rect.size.width / 2.0) as f32,
        radiusY: (rect.size.height / 2.0) as f32,
    }
}

impl<'a> DrawingContext<'a> {
    pub fn new(
        d2d: ID2D1Factory,
        dwrite: IDWriteFactory,
        target: ID2D1RenderTarget,
        owner: Option<&'a mut dyn ContextOwner>,
    ) -> Self {
        Self {
            d2d,
            dwrite,
            target,
            owner,
            ended: false,
        }
    }

    pub fn render_target(&self) -> &ID2D1RenderTarget {
        &self.target
    }

    fn end_draw(&mut self) -> Result<()> {
        if self.ended {
            return Ok(());
        }
        self.ended = true;
        match self.owner.take() {
            Some(owner) => owner.end_draw(&self.target),
            None => {
                unsafe { self.target.EndDraw(None, None).ok()? };
                Ok(())
            }
        }
    }

    pub fn close(mut self) -> Result<()> {
        self.end_draw()
    }

    #[inline]
    fn get_brush(&self, brush: impl Brush, rect: Rect) -> Result<ID2D1Brush> {
        brush.create(&self.target, to_trans(rect))
    }

    #[inline]
    fn get_pen(&self, pen: impl Pen, rect: Rect) -> Result<(ID2D1Brush, f32)> {
        pen.create(&self.target, to_trans(rect))
    }

    fn get_arc_geo(&self, rect: Rect, start: f64, end: f64, close: bool) -> Result<ID2D1Geometry> {
        unsafe {
            let geo = self.d2d.CreatePathGeometry()?;
            let sink = geo.Open()?;
            let (radius, centerp, startp, endp) = get_arc(rect, start, end);
            sink.BeginFigure(point_2f(startp), D2D1_FIGURE_BEGIN_HOLLOW);
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: point_2f(endp),
                size: size_f(radius),
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: if (end - start) > std::f64::consts::PI {
                    D2D1_ARC_SIZE_LARGE
                } else {
                    D2D1_ARC_SIZE_SMALL
                },
            });
            if close {
                sink.AddLine(point_2f(centerp));
            }
            sink.EndFigure(if close {
                D2D1_FIGURE_END_CLOSED
            } else {
                D2D1_FIGURE_END_OPEN
            });
            sink.Close()?;
            geo.cast()
        }
    }

    fn get_str_layout(
        &self,
        font: Font,
        anchor: RelativePoint,
        pos: Point,
        s: &str,
    ) -> Result<(Rect, IDWriteTextLayout)> {
        unsafe {
            let f = U16CString::from_str_truncate(&font.family);
            let format = self.dwrite.CreateTextFormat(
                windows_core::PCWSTR::from_raw(f.as_ptr()),
                None,
                if font.bold {
                    DWRITE_FONT_WEIGHT_BOLD
                } else {
                    DWRITE_FONT_WEIGHT_NORMAL
                },
                if font.italic {
                    DWRITE_FONT_STYLE_ITALIC
                } else {
                    DWRITE_FONT_STYLE_NORMAL
                },
                DWRITE_FONT_STRETCH_NORMAL,
                font.size as f32,
                windows_core::w!(""),
            )?;
            let size = self.target.GetSize();
            let s = U16CString::from_str_truncate(s);
            let layout =
                self.dwrite
                    .CreateTextLayout(s.as_slice(), &format, size.width, size.height)?;
            let mut metrics = MaybeUninit::uninit();
            layout.GetMetrics(metrics.as_mut_ptr())?;
            let metrics = metrics.assume_init();
            let x = pos.x - metrics.width as f64 * anchor.x;
            let y = pos.y - metrics.height as f64 * anchor.y;
            let size = Size::new(metrics.width as f64, metrics.height as f64);
            let rect = Rect::new(Point::new(x, y), size);
            Ok((rect, layout))
        }
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<()> {
        unsafe {
            let matrix = matrix_f(transform);
            self.target.SetTransform(&matrix);
        }
        Ok(())
    }

    pub fn transform(&self) -> Result<Transform> {
        let mut matrix = MaybeUninit::uninit();
        let matrix = unsafe {
            self.target.GetTransform(matrix.as_mut_ptr());
            matrix.assume_init()
        };
        Ok(unsafe {
            Transform::new(
                matrix.Anonymous.Anonymous2._11 as f64,
                matrix.Anonymous.Anonymous2._12 as f64,
                matrix.Anonymous.Anonymous2._21 as f64,
                matrix.Anonymous.Anonymous2._22 as f64,
                matrix.Anonymous.Anonymous2._31 as f64,
                matrix.Anonymous.Anonymous2._32 as f64,
            )
        })
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) -> Result<()> {
        let width = pen.width();
        let rect = unsafe {
            path.geo
                .GetWidenedBounds(width, None, None, D2D1_DEFAULT_FLATTENING_TOLERANCE)?
        };
        let (b, width) = self.get_pen(
            pen,
            RectBox::new(
                Point::new(rect.left as _, rect.top as _),
                Point::new(rect.right as _, rect.bottom as _),
            )
            .to_rect(),
        )?;
        unsafe {
            self.target.DrawGeometry(&path.geo, &b, width, None);
        }
        Ok(())
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) -> Result<()> {
        let rect = unsafe { path.geo.GetBounds(None)? };
        let b = self.get_brush(
            brush,
            RectBox::new(
                Point::new(rect.left as _, rect.top as _),
                Point::new(rect.right as _, rect.bottom as _),
            )
            .to_rect(),
        )?;
        unsafe {
            self.target.FillGeometry(&path.geo, &b, None);
        }
        Ok(())
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let geo = self.get_arc_geo(rect, start, end, false)?;
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawGeometry(&geo, &b, width, None);
        }
        Ok(())
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let geo = self.get_arc_geo(rect, start, end, true)?;
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawGeometry(&geo, &b, width, None);
        }
        Ok(())
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> Result<()> {
        let geo = self.get_arc_geo(rect, start, end, true)?;
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillGeometry(&geo, &b, None);
        }
        Ok(())
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let e = ellipse(rect);
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawEllipse(&e, &b, width, None);
        }
        Ok(())
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let e = ellipse(rect);
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillEllipse(&e, &b);
        }
        Ok(())
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) -> Result<()> {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target
                .DrawLine(point_2f(start), point_2f(end), &b, width, None);
        }
        Ok(())
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawRectangle(&rect_f(rect), &b, width, None);
        }
        Ok(())
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillRectangle(&rect_f(rect), &b);
        }
        Ok(())
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) -> Result<()> {
        let (b, width) = self.get_pen(pen, rect)?;
        unsafe {
            self.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: rect_f(rect),
                    radiusX: round.width as f32,
                    radiusY: round.height as f32,
                },
                &b,
                width,
                None,
            );
        }
        Ok(())
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) -> Result<()> {
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: rect_f(rect),
                    radiusX: round.width as f32,
                    radiusY: round.height as f32,
                },
                &b,
            );
        }
        Ok(())
    }

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: Font,
        anchor: RelativePoint,
        pos: Point,
        text: &str,
    ) -> Result<()> {
        let (rect, layout) = self.get_str_layout(font, anchor, pos, text.as_ref())?;
        let b = self.get_brush(brush, rect)?;
        unsafe {
            self.target.DrawTextLayout(
                point_2f(rect.origin),
                &layout,
                &b,
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            );
        }
        Ok(())
    }

    pub fn measure_str(&self, font: Font, text: &str) -> Result<Size> {
        let (rect, _) =
            self.get_str_layout(font, RelativePoint::zero(), Point::zero(), text.as_ref())?;
        Ok(rect.size)
    }

    pub fn create_image(&self, image: Cow<'_, DynamicImage>) -> Result<DrawingImage> {
        DrawingImage::from_image(&self.target, image)
    }

    pub fn create_image_from_premultiplied_bgra8(
        &self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<DrawingImage> {
        DrawingImage::from_premultiplied_bgra8(&self.target, width, height, pixels)
    }

    pub fn draw_image(
        &mut self,
        image: &DrawingImage,
        rect: Rect,
        clip: Option<BitmapRect>,
    ) -> Result<()> {
        unsafe {
            let clip = clip.map(bitmap_rect_f);
            self.target.DrawBitmap(
                &*image.get_bitmap(&self.target)?,
                Some(&rect_f(rect)),
                1.0,
                D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                clip.as_ref().map(|r| r as *const _),
            );
        }
        Ok(())
    }

    pub fn create_path_builder(&self, start: Point) -> Result<DrawingPathBuilder> {
        DrawingPathBuilder::new(&self.d2d, start)
    }
}

pub struct DrawingPath {
    geo: ID2D1Geometry,
}

impl DrawingPath {
    fn new(geo: ID2D1Geometry) -> Self {
        Self { geo }
    }
}

pub struct DrawingPathBuilder {
    geo: ID2D1PathGeometry,
    sink: ID2D1GeometrySink,
}

impl DrawingPathBuilder {
    fn new(d2d: &ID2D1Factory, start: Point) -> Result<Self> {
        unsafe {
            let geo = d2d.CreatePathGeometry()?;
            let sink = geo.Open()?;
            sink.BeginFigure(point_2f(start), D2D1_FIGURE_BEGIN_HOLLOW);
            Ok(Self { geo, sink })
        }
    }

    pub fn add_line(&mut self, p: Point) -> Result<()> {
        unsafe {
            self.sink.AddLine(point_2f(p));
        }
        Ok(())
    }

    pub fn add_arc(
        &mut self,
        center: Point,
        radius: Size,
        start: f64,
        end: f64,
        clockwise: bool,
    ) -> Result<()> {
        unsafe {
            let startp =
                center + Vector::new(radius.width * start.cos(), radius.height * start.sin());
            let endp = center + Vector::new(radius.width * end.cos(), radius.height * end.sin());
            self.add_line(startp)?;
            self.sink.AddArc(&D2D1_ARC_SEGMENT {
                point: point_2f(endp),
                size: size_f(radius),
                rotationAngle: 0.0,
                sweepDirection: if clockwise {
                    D2D1_SWEEP_DIRECTION_CLOCKWISE
                } else {
                    D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE
                },
                arcSize: if (end - start) > std::f64::consts::PI {
                    D2D1_ARC_SIZE_LARGE
                } else {
                    D2D1_ARC_SIZE_SMALL
                },
            });
        }
        Ok(())
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) -> Result<()> {
        unsafe {
            self.sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: point_2f(p1),
                point2: point_2f(p2),
                point3: point_2f(p3),
            });
        }
        Ok(())
    }

    pub fn build(self, close: bool) -> Result<DrawingPath> {
        unsafe {
            self.sink.EndFigure(if close {
                D2D1_FIGURE_END_CLOSED
            } else {
                D2D1_FIGURE_END_OPEN
            });
            self.sink.Close()?;
            Ok(DrawingPath::new(self.geo.cast()?))
        }
    }
}

const MATRIX_IDENTITY: D2D_MATRIX_3X2_F = matrix_f(Transform::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));

const BRUSH_PROPERTIES_DEFAULT: D2D1_BRUSH_PROPERTIES = D2D1_BRUSH_PROPERTIES {
    opacity: 1.0,
    transform: MATRIX_IDENTITY,
};

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> Result<ID2D1Brush>;
}

impl<B: Brush> Brush for &'_ B {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> Result<ID2D1Brush> {
        (**self).create(target, trans)
    }
}

impl Brush for SolidColorBrush {
    fn create(&self, target: &ID2D1RenderTarget, _trans: RelativeToLogical) -> Result<ID2D1Brush> {
        unsafe {
            target
                .CreateSolidColorBrush(&color_f(self.color), Some(&BRUSH_PROPERTIES_DEFAULT))?
                .cast()
        }
    }
}

impl Brush for LinearGradientBrush {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> Result<ID2D1Brush> {
        let props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
            startPoint: point_2f(trans.transform_point(self.start)),
            endPoint: point_2f(trans.transform_point(self.end)),
        };
        let stops = self.stops.iter().map(gradient_stop).collect::<Vec<_>>();
        unsafe {
            let stop_collection = target.CreateGradientStopCollection(
                &stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;
            target
                .CreateLinearGradientBrush(
                    &props,
                    Some(&BRUSH_PROPERTIES_DEFAULT),
                    &stop_collection,
                )?
                .cast()
        }
    }
}

impl Brush for RadialGradientBrush {
    fn create(&self, target: &ID2D1RenderTarget, trans: RelativeToLogical) -> Result<ID2D1Brush> {
        let radius = self.radius.to_vector();
        let radius = trans.transform_vector(radius);
        let props = D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
            center: point_2f(trans.transform_point(self.center)),
            gradientOriginOffset: point_2f(
                trans.transform_vector(self.origin - self.center).to_point(),
            ),
            radiusX: radius.x as f32,
            radiusY: radius.y as f32,
        };
        let stops = self.stops.iter().map(gradient_stop).collect::<Vec<_>>();
        unsafe {
            let stop_collection = target.CreateGradientStopCollection(
                &stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;
            target
                .CreateRadialGradientBrush(
                    &props,
                    Some(&BRUSH_PROPERTIES_DEFAULT),
                    &stop_collection,
                )?
                .cast()
        }
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToLogical,
    ) -> Result<(ID2D1Brush, f32)>;
    #[doc(hidden)]
    fn width(&self) -> f32;
}

impl<P: Pen> Pen for &'_ P {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToLogical,
    ) -> Result<(ID2D1Brush, f32)> {
        (**self).create(target, trans)
    }

    fn width(&self) -> f32 {
        (**self).width()
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create(
        &self,
        target: &ID2D1RenderTarget,
        trans: RelativeToLogical,
    ) -> Result<(ID2D1Brush, f32)> {
        let brush = self.brush.create(target, trans)?;
        Ok((brush, self.width as _))
    }

    fn width(&self) -> f32 {
        self.width as _
    }
}

pub struct DrawingImage {
    bitmap: IWICBitmap,
    cache: RefCell<Option<DrawingImageCache>>,
}

struct DrawingImageCache {
    target: ID2D1RenderTarget,
    bitmap: ID2D1Bitmap,
}

impl DrawingImage {
    /// Create an image filled with transparent pixels.
    pub fn new(size: BitmapSize) -> Result<Self> {
        let width = size.width as u32;
        let height = size.height as u32;
        let bitmap = crate::runtime::with_wic_factory(|wic| unsafe {
            wic.CreateBitmap(
                width,
                height,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapCacheOnLoad,
            )
        })?;
        unsafe {
            let lock = bitmap.Lock(std::ptr::null(), WICBitmapLockWrite as u32)?;
            let mut size = 0;
            let mut data = std::ptr::null_mut();
            lock.GetDataPointer(&mut size, &mut data)?;
            std::ptr::write_bytes(data, 0, size as usize);
        }
        Self::from_bitmap(None, bitmap)
    }

    fn from_image(target: &ID2D1RenderTarget, image: Cow<'_, DynamicImage>) -> Result<Self> {
        let image = match image {
            Cow::Owned(image) => image.into_rgba8(),
            Cow::Borrowed(image) => image.to_rgba8(),
        };
        let (width, height) = image.dimensions();
        let mut pixels = image.into_raw();
        rgba8_to_pbgra8(&mut pixels);
        Self::from_premultiplied_bgra8(target, width, height, &pixels)
    }

    /// Create a [`DrawingImage`] from premultiplied BGRA pixels.
    pub fn from_premultiplied_bgra8(
        target: &ID2D1RenderTarget,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<Self> {
        let bitmap = crate::runtime::with_wic_factory(|wic| unsafe {
            wic.CreateBitmapFromMemory(
                width,
                height,
                &GUID_WICPixelFormat32bppPBGRA,
                width * 4,
                pixels,
            )
        })?;
        Self::from_bitmap(Some(target), bitmap)
    }

    fn from_bitmap(target: Option<&ID2D1RenderTarget>, bitmap: IWICBitmap) -> Result<Self> {
        let cache = target
            .map(|target| -> Result<DrawingImageCache> {
                Ok(DrawingImageCache {
                    target: target.clone(),
                    bitmap: Self::create_bitmap(target, &bitmap)?,
                })
            })
            .transpose()?;
        Ok(Self {
            bitmap,
            cache: RefCell::new(cache),
        })
    }

    fn create_bitmap(target: &ID2D1RenderTarget, bitmap: &IWICBitmap) -> Result<ID2D1Bitmap> {
        // Direct2D bitmap coordinates are DIPs, but winio measures images and
        // clip regions in pixels (like the other backends). A 96 DPI bitmap
        // has pixel-sized DIPs.
        let prop = D2D1_BITMAP_PROPERTIES {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
        };
        unsafe { target.CreateBitmapFromWicBitmap(&**bitmap, Some(&prop)) }
    }

    fn recreate(&self, target: &ID2D1RenderTarget) -> Result<()> {
        let bitmap = Self::create_bitmap(target, &self.bitmap)?;
        *self.cache.borrow_mut() = Some(DrawingImageCache {
            target: target.clone(),
            bitmap,
        });
        Ok(())
    }

    pub fn get_bitmap(&self, target: &ID2D1RenderTarget) -> Result<Ref<'_, ID2D1Bitmap>> {
        if self
            .cache
            .borrow()
            .as_ref()
            .map(|cache| cache.target.as_raw())
            != Some(target.as_raw())
        {
            self.recreate(target)?;
        }
        Ok(Ref::map(self.cache.borrow(), |cache| {
            &cache.as_ref().expect("bitmap is created").bitmap
        }))
    }

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        let prop = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        let d2d = d2d1_factory()?;
        let dwrite = dwrite_factory()?;
        let target = unsafe { d2d.CreateWicBitmapRenderTarget(&self.bitmap, &prop)? };
        // The cached D2D bitmap no longer matches the WIC bitmap content.
        *self.cache.borrow_mut() = None;
        Ok(DrawingContext::new(
            d2d.clone().into(),
            dwrite.clone(),
            target,
            None,
        ))
    }

    /// Copy the premultiplied BGRA pixels into `buffer`.
    pub fn copy_pixels(&self, buffer: &mut [u8]) -> Result<()> {
        let (width, _) = self.dimensions()?;
        unsafe {
            self.bitmap
                .CopyPixels(None, width * 4, buffer.len() as u32, buffer.as_mut_ptr())?;
        }
        Ok(())
    }

    pub fn to_dynamic_image(&self) -> Result<DynamicImage> {
        let (width, height) = self.dimensions()?;
        let mut pixels = vec![0; (width * height * 4) as usize];
        unsafe {
            self.bitmap
                .CopyPixels(None, width * 4, pixels.len() as u32, pixels.as_mut_ptr())?;
        }
        pbgra8_to_rgba8(&mut pixels);
        Ok(DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, pixels).expect("invalid image buffer"),
        ))
    }

    pub fn size(&self) -> Result<BitmapSize> {
        let (width, height) = self.dimensions()?;
        Ok(BitmapSize::new(width as usize, height as usize))
    }

    fn dimensions(&self) -> Result<(u32, u32)> {
        let mut width = 0;
        let mut height = 0;
        unsafe { self.bitmap.GetSize(&mut width, &mut height)? };
        Ok((width, height))
    }
}

impl TryFrom<&DrawingImage> for DynamicImage {
    type Error = Error;

    fn try_from(value: &DrawingImage) -> Result<Self> {
        value.to_dynamic_image()
    }
}

impl TryFrom<DrawingImage> for DynamicImage {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        Self::try_from(&value)
    }
}
