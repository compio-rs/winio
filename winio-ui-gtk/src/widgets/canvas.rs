use std::{
    borrow::Cow,
    cell::RefCell,
    f64::consts::{FRAC_PI_2, PI},
    mem::size_of,
    rc::Rc,
};

use compio_log::error;
use gtk4::{
    EventControllerMotion, EventControllerScroll, EventControllerScrollFlags, GestureClick,
    cairo::{
        Content, Context, Format, ImageSurface, LinearGradient, Matrix, RadialGradient,
        RecordingSurface,
    },
    gdk::{self, ScrollUnit, Texture},
    gdk_pixbuf::Pixbuf,
    glib::{Propagation, object::Cast},
    pango::{
        Context as PangoContext, FontDescription, Layout, SCALE as PANGO_SCALE, Style, Weight,
    },
    prelude::{DrawingAreaExtManual, EventControllerExt, GestureSingleExt, WidgetExt},
};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage};
use inherit_methods_macro::inherit_methods;
use pangocairo::functions::show_layout;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{
    BrushPen, Font, KeyCode, LinearGradientBrush, MouseButton, Point, RadialGradientBrush, Rect,
    RectBox, RelativePoint, RelativeToLogical, Size, SolidColorBrush, Transform, Vector,
    packed_rows, premultiply_rgba_f32, unpremultiply_rgba_f32,
};

use crate::{Error, GlobalRuntime, Image, Result, platform::Keyboard, widgets::Widget};

#[derive(Debug)]
pub struct Canvas {
    on_motion: Rc<Callback<Point>>,
    on_pressed: Rc<Callback<MouseButton>>,
    on_released: Rc<Callback<MouseButton>>,
    on_scroll: Rc<Callback<Vector>>,
    keyboard: Keyboard,
    widget: gtk4::DrawingArea,
    handle: Widget,
    surface: Rc<RefCell<RecordingSurface>>,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::DrawingArea::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let keyboard = Keyboard::new(&widget);

        let on_motion = Rc::new(Callback::new());
        let on_pressed = Rc::new(Callback::new());
        let on_released = Rc::new(Callback::new());
        let on_scroll = Rc::new(Callback::new());

        let surface = Rc::new(RefCell::new(RecordingSurface::create(
            Content::ColorAlpha,
            None,
        )?));

        widget.set_draw_func({
            let surface = surface.clone();
            move |_, ctx, _, _| {
                if let Err(_e) = (|| {
                    ctx.set_source_surface(&*surface.borrow(), 0.0, 0.0)?;
                    ctx.paint()
                })() {
                    error!("Canvas draw error: {_e:?}");
                }
            }
        });

        let controller = EventControllerMotion::new();
        controller.connect_motion({
            let on_motion = on_motion.clone();
            move |_, x, y| {
                on_motion.signal::<GlobalRuntime>(Point::new(x, y));
            }
        });
        widget.add_controller(controller);

        const fn gtk_current_button(b: u32) -> MouseButton {
            match b {
                1 => MouseButton::Left,
                2 => MouseButton::Middle,
                3 => MouseButton::Right,
                _ => MouseButton::Other,
            }
        }

        let controller = GestureClick::new();
        controller.connect_pressed({
            let on_pressed = on_pressed.clone();
            move |controller, _, _, _| {
                if let Some(widget) = controller.widget() {
                    widget.grab_focus();
                }
                on_pressed.signal::<GlobalRuntime>(gtk_current_button(controller.current_button()));
            }
        });
        controller.connect_released({
            let on_released = on_released.clone();
            move |controller, _, _, _| {
                on_released
                    .signal::<GlobalRuntime>(gtk_current_button(controller.current_button()));
            }
        });
        widget.add_controller(controller);

        let controller = EventControllerScroll::new(EventControllerScrollFlags::BOTH_AXES);
        controller.connect_scroll({
            let on_scroll = on_scroll.clone();
            move |controller, dx, dy| {
                let scale = match controller.unit() {
                    ScrollUnit::Wheel => 120.0,
                    _ => 1.0,
                };
                on_scroll.signal::<GlobalRuntime>(Vector::new(dx, -dy) * scale);
                Propagation::Stop
            }
        });
        widget.add_controller(controller);

        Ok(Self {
            on_motion,
            on_pressed,
            on_released,
            on_scroll,
            keyboard,
            widget,
            handle,
            surface,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        let surface = RecordingSurface::create(Content::ColorAlpha, None)?;
        let ctx = Context::new(&surface)?;
        let pango = self.widget.pango_context();
        Ok(DrawingContext {
            surface: Some(surface),
            ctx,
            pango,
            target: ContextTarget::Canvas(self),
        })
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.on_pressed.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.on_released.wait().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.on_motion.wait().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.on_scroll.wait().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.keyboard.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.keyboard.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.keyboard.wait_key_char().await
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

pub struct DrawingContext<'a> {
    surface: Option<RecordingSurface>,
    ctx: Context,
    pango: PangoContext,
    target: ContextTarget<'a>,
}

enum ContextTarget<'a> {
    Canvas(&'a mut Canvas),
    /// Keeps the image alive while the context is active.
    Image(#[allow(dead_code)] &'a mut DrawingImage),
}

impl ContextTarget<'_> {
    fn width(&self) -> i32 {
        match self {
            ContextTarget::Canvas(canvas) => canvas.widget.width(),
            ContextTarget::Image(image) => image.0.width(),
        }
    }
}

#[inline]
fn to_trans(mut rect: Rect) -> RelativeToLogical {
    if rect.size.width == 0.0 {
        rect.size.width = 0.1;
    }
    if rect.size.height == 0.0 {
        rect.size.height = 0.1;
    }
    RelativeToLogical::scale(rect.size.width, rect.size.height)
        .then_translate(rect.origin.to_vector())
}

impl DrawingContext<'_> {
    pub fn set_transform(&mut self, transform: Transform) -> Result<()> {
        self.ctx.set_matrix(Matrix::new(
            transform.m11,
            transform.m12,
            transform.m21,
            transform.m22,
            transform.m31,
            transform.m32,
        ));
        Ok(())
    }

    pub fn transform(&self) -> Result<Transform> {
        let m = self.ctx.matrix();
        Ok(Transform::new(
            m.xx(),
            m.yx(),
            m.xy(),
            m.yy(),
            m.x0(),
            m.y0(),
        ))
    }

    #[inline]
    fn set_brush(&self, brush: impl Brush, rect: Rect) -> Result<()> {
        brush.set(&self.ctx, to_trans(rect))
    }

    #[inline]
    fn set_pen(&self, pen: impl Pen, rect: Rect) -> Result<()> {
        pen.set(&self.ctx, to_trans(rect))
    }

    fn path_arc(&self, rect: Rect, start: f64, end: f64, pie: bool) {
        let save_matrix = self.ctx.matrix();
        let rate = rect.size.height / rect.size.width;
        self.ctx.scale(1.0, rate);
        self.ctx.new_path();
        let center = rect.center();
        if pie {
            self.ctx.move_to(center.x, center.y / rate);
        }
        self.ctx
            .arc(center.x, center.y / rate, rect.size.width / 2.0, start, end);
        if pie {
            self.ctx.close_path();
        }
        self.ctx.set_matrix(save_matrix);
    }

    fn path_rect(&self, rect: Rect) {
        self.ctx.new_path();
        self.ctx.rectangle(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
    }

    fn path_round_rect(&self, rect: Rect, round: Size) {
        let save_matrix = self.ctx.matrix();
        self.ctx.scale(1.0, round.height / round.width);
        self.ctx.new_sub_path();
        self.ctx.arc(
            rect.origin.x + rect.size.width - round.width,
            rect.origin.y + round.height,
            round.width,
            -FRAC_PI_2,
            0.0,
        );
        self.ctx.arc(
            rect.origin.x + rect.size.width - round.width,
            rect.origin.y + rect.size.height - round.height,
            round.width,
            0.0,
            FRAC_PI_2,
        );
        self.ctx.arc(
            rect.origin.x + round.width,
            rect.origin.y + rect.size.height - round.height,
            round.width,
            FRAC_PI_2,
            PI,
        );
        self.ctx.arc(
            rect.origin.x + round.width,
            rect.origin.y + round.height,
            round.width,
            PI,
            FRAC_PI_2 * 3.0,
        );
        self.ctx.close_path();
        self.ctx.set_matrix(save_matrix);
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) -> Result<()> {
        let (x, y, width, height) = path.surface.ink_extents();
        let rect = Rect::new(Point::new(x, y), Size::new(width, height));
        pen.set(&path.ctx, to_trans(rect))?;
        path.ctx.stroke()?;
        self.ctx.set_source_surface(&path.surface, 0.0, 0.0)?;
        self.ctx.paint()?;
        Ok(())
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) -> Result<()> {
        let (x, y, width, height) = path.surface.ink_extents();
        let rect = Rect::new(Point::new(x, y), Size::new(width, height));
        brush.set(&path.ctx, to_trans(rect))?;
        path.ctx.stroke()?;
        self.ctx.set_source_surface(&path.surface, 0.0, 0.0)?;
        self.ctx.paint()?;
        Ok(())
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        self.path_arc(rect, start, end, false);
        self.set_pen(pen, rect)?;
        self.ctx.stroke()?;
        Ok(())
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        self.path_arc(rect, start, end, true);
        self.set_pen(pen, rect)?;
        self.ctx.stroke()?;
        Ok(())
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> Result<()> {
        self.path_arc(rect, start, end, true);
        self.set_brush(brush, rect)?;
        self.ctx.fill()?;
        Ok(())
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        self.draw_arc(pen, rect, 0.0, PI * 2.0)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        self.fill_pie(brush, rect, 0.0, PI * 2.0)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) -> Result<()> {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        self.ctx.new_path();
        self.ctx.move_to(start.x, start.y);
        self.ctx.line_to(end.x, end.y);
        self.set_pen(pen, rect)?;
        self.ctx.stroke()?;
        Ok(())
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        self.path_rect(rect);
        self.set_pen(pen, rect)?;
        self.ctx.stroke()?;
        Ok(())
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        self.path_rect(rect);
        self.set_brush(brush, rect)?;
        self.ctx.fill()?;
        Ok(())
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) -> Result<()> {
        self.path_round_rect(rect, round);
        self.set_pen(pen, rect)?;
        self.ctx.stroke()?;
        Ok(())
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) -> Result<()> {
        self.path_round_rect(rect, round);
        self.set_brush(brush, rect)?;
        self.ctx.fill()?;
        Ok(())
    }

    fn measure_str_impl(&self, font: &Font, text: &str) -> (Size, Layout) {
        let layout = Layout::new(&self.pango);
        layout.set_text(text);
        let mut desp = FontDescription::from_string(&font.family);
        desp.set_size((font.size / 1.33) as i32 * PANGO_SCALE);
        if font.italic {
            desp.set_style(Style::Italic);
        }
        if font.bold {
            desp.set_weight(Weight::Bold);
        }
        layout.set_font_description(Some(&desp));
        layout.set_width(self.target.width() * PANGO_SCALE);

        let (width, height) = layout.pixel_size();
        (Size::new(width as f64, height as f64), layout)
    }

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: Font,
        anchor: RelativePoint,
        pos: Point,
        text: &str,
    ) -> Result<()> {
        let (size, layout) = self.measure_str_impl(&font, text);
        let (width, height) = (size.width, size.height);

        let x = pos.x - width * anchor.x;
        let y = pos.y - height * anchor.y;
        let rect = Rect::new(Point::new(x, y), Size::new(width, height));

        self.ctx.move_to(rect.origin.x, rect.origin.y);
        self.set_brush(brush, rect)?;
        show_layout(&self.ctx, &layout);
        Ok(())
    }

    pub fn measure_str(&self, font: Font, text: &str) -> Result<Size> {
        Ok(self.measure_str_impl(&font, text).0)
    }

    pub fn create_image(&self, image: Cow<'_, DynamicImage>) -> Result<DrawingImage> {
        DrawingImage::new(image)
    }

    pub fn create_image_empty(&self, size: Size) -> Result<DrawingImage> {
        DrawingImage::new_empty(size)
    }

    pub fn draw_image(
        &mut self,
        image: &DrawingImage,
        rect: Rect,
        clip: Option<Rect>,
    ) -> Result<()> {
        self.ctx.save()?;

        let clip = if let Some(clip) = clip {
            self.ctx.rectangle(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
            self.ctx.clip();
            clip
        } else {
            image.size()?.into()
        };

        self.ctx.new_path();
        let scalex = rect.width() / clip.width();
        let scaley = rect.height() / clip.height();
        self.ctx.translate(rect.origin.x, rect.origin.y);
        self.ctx.scale(scalex, scaley);
        self.ctx
            .set_source_surface(&image.0, -clip.origin.x, -clip.origin.y)?;
        self.ctx.paint()?;
        self.ctx.restore()?;
        Ok(())
    }

    pub fn create_path_builder(&self, start: Point) -> Result<DrawingPathBuilder> {
        DrawingPathBuilder::new(start)
    }

    pub fn close(self) -> Result<()> {
        Ok(())
    }
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let ContextTarget::Canvas(canvas) = &mut self.target {
            *canvas.surface.borrow_mut() =
                self.surface.take().expect("DrawingContext dropped twice");
            canvas.widget.queue_draw();
        }
    }
}

pub type DrawingPath = DrawingPathBuilder;

pub struct DrawingPathBuilder {
    surface: RecordingSurface,
    ctx: Context,
}

impl DrawingPathBuilder {
    fn new(start: Point) -> Result<Self> {
        let surface = RecordingSurface::create(Content::ColorAlpha, None)?;
        let ctx = Context::new(&surface)?;
        ctx.new_path();
        ctx.move_to(start.x, start.y);
        Ok(Self { surface, ctx })
    }

    pub fn add_line(&mut self, p: Point) -> Result<()> {
        self.ctx.line_to(p.x, p.y);
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
        let save_matrix = self.ctx.matrix();
        let rate = radius.height / radius.width;
        self.ctx.scale(1.0, rate);
        if clockwise {
            self.ctx
                .arc(center.x, center.y / rate, radius.width, start, end);
        } else {
            self.ctx
                .arc_negative(center.x, center.y / rate, radius.width, start, end);
        }
        self.ctx.set_matrix(save_matrix);
        Ok(())
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) -> Result<()> {
        self.ctx.curve_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
        Ok(())
    }

    pub fn build(self, close: bool) -> Result<DrawingPath> {
        if close {
            self.ctx.close_path();
        }
        Ok(self)
    }
}

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()>;
}

impl<B: Brush> Brush for &'_ B {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()> {
        (**self).set(ctx, trans)
    }
}

impl Brush for SolidColorBrush {
    fn set(&self, ctx: &Context, _trans: RelativeToLogical) -> Result<()> {
        ctx.set_source_rgba(
            self.color.r as f64 / 255.0,
            self.color.g as f64 / 255.0,
            self.color.b as f64 / 255.0,
            self.color.a as f64 / 255.0,
        );
        Ok(())
    }
}

impl Brush for LinearGradientBrush {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()> {
        let start = trans.transform_point(self.start);
        let end = trans.transform_point(self.end);
        let p = LinearGradient::new(start.x, start.y, end.x, end.y);
        for stop in &self.stops {
            p.add_color_stop_rgba(
                stop.pos,
                stop.color.r as f64 / 255.0,
                stop.color.g as f64 / 255.0,
                stop.color.b as f64 / 255.0,
                stop.color.a as f64 / 255.0,
            );
        }
        ctx.set_source(&p)?;
        Ok(())
    }
}

impl Brush for RadialGradientBrush {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()> {
        let trans = trans.then_scale(1.0, self.radius.height / self.radius.width);
        let p = RadialGradient::new(
            self.origin.x,
            self.origin.y,
            0.0,
            self.center.x,
            self.center.y,
            self.radius.width,
        );
        p.set_matrix(Matrix::new(
            trans.m11, trans.m12, trans.m21, trans.m22, trans.m31, trans.m32,
        ));
        for stop in &self.stops {
            p.add_color_stop_rgba(
                stop.pos,
                stop.color.r as f64 / 255.0,
                stop.color.g as f64 / 255.0,
                stop.color.b as f64 / 255.0,
                stop.color.a as f64 / 255.0,
            );
        }
        ctx.set_source(&p)?;
        Ok(())
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()>;
}

impl<P: Pen> Pen for &'_ P {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()> {
        (**self).set(ctx, trans)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) -> Result<()> {
        self.brush.set(ctx, trans)?;
        ctx.set_line_width(self.width);
        Ok(())
    }
}

pub struct DrawingImage(ImageSurface);

impl DrawingImage {
    fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        const CAIRO_FORMAT_RGB96F: Format = Format::__Unknown(6);
        const CAIRO_FORMAT_RGBA128F: Format = Format::__Unknown(7);

        let width = image.width();
        let height = image.height();
        let (format, buffer): (Format, F32Buffer) = match image {
            Cow::Owned(image) => match image {
                DynamicImage::ImageRgb32F(image) => {
                    (CAIRO_FORMAT_RGB96F, F32Buffer(image.into_raw()))
                }
                DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgb16(_) => (
                    CAIRO_FORMAT_RGB96F,
                    F32Buffer(image.into_rgb32f().into_raw()),
                ),
                DynamicImage::ImageRgba32F(image) => {
                    (CAIRO_FORMAT_RGBA128F, premultiplied(image.into_raw()))
                }
                _ => (
                    CAIRO_FORMAT_RGBA128F,
                    premultiplied(image.into_rgba32f().into_raw()),
                ),
            },
            Cow::Borrowed(image) => match image {
                DynamicImage::ImageRgb32F(_) => {
                    (CAIRO_FORMAT_RGB96F, F32Buffer(image.to_rgb32f().into_raw()))
                }
                DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgb16(_) => {
                    (CAIRO_FORMAT_RGB96F, F32Buffer(image.to_rgb32f().into_raw()))
                }
                DynamicImage::ImageRgba32F(_) => (
                    CAIRO_FORMAT_RGBA128F,
                    premultiplied(image.to_rgba32f().into_raw()),
                ),
                _ => (
                    CAIRO_FORMAT_RGBA128F,
                    premultiplied(image.to_rgba32f().into_raw()),
                ),
            },
        };
        let stride = format.stride_for_width(width)?;
        let surface =
            ImageSurface::create_for_data(buffer, format, width as _, height as _, stride as _)?;
        Ok(Self(surface))
    }

    #[allow(deprecated)]
    pub(crate) fn from_texture(texture: &Texture) -> Result<Self> {
        let pixbuf = gdk::pixbuf_get_from_texture(texture).ok_or(Error::NullPointer)?;
        Self::new(Cow::Owned(pixbuf_to_dynamic_image(&pixbuf)?))
    }

    /// Create an empty image filled with transparent pixels.
    pub(crate) fn new_empty(size: Size) -> Result<Self> {
        const CAIRO_FORMAT_RGBA128F: Format = Format::__Unknown(7);

        let width = size.width.round().max(1.0) as i32;
        let height = size.height.round().max(1.0) as i32;
        let stride = CAIRO_FORMAT_RGBA128F.stride_for_width(width as _)?;
        let buffer = F32Buffer::zeroed(stride as usize * height as usize);
        let surface =
            ImageSurface::create_for_data(buffer, CAIRO_FORMAT_RGBA128F, width, height, stride)?;
        Ok(Self(surface))
    }

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        let ctx = Context::new(&self.0)?;
        Ok(DrawingContext {
            surface: None,
            ctx,
            pango: PangoContext::new(),
            target: ContextTarget::Image(self),
        })
    }

    #[allow(deprecated)]
    fn to_texture(&self) -> Result<Texture> {
        let pixbuf = self.surface_pixbuf()?;
        Ok(Texture::for_pixbuf(&pixbuf))
    }

    fn to_dynamic_image(&self) -> Result<DynamicImage> {
        const CAIRO_FORMAT_RGB96F: Format = Format::__Unknown(6);
        const CAIRO_FORMAT_RGBA128F: Format = Format::__Unknown(7);

        let width = self.0.width() as u32;
        let height = self.0.height() as u32;
        let stride = self.0.stride() as usize;
        let (row, alpha) = match self.0.format() {
            CAIRO_FORMAT_RGB96F => (width as usize * 12, false),
            CAIRO_FORMAT_RGBA128F => (width as usize * 16, true),
            _ => return Err(Error::NotSupported),
        };
        let mut pixels = None;
        self.0.with_data(|data| {
            pixels = Some(packed_rows::<f32>(data, stride, row, height as usize));
        })?;
        let mut pixels = pixels.ok_or(Error::NotSupported)?;
        if alpha {
            unpremultiply_rgba_f32(&mut pixels);
            Ok(DynamicImage::ImageRgba32F(
                ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(width, height, pixels)
                    .expect("invalid image buffer"),
            ))
        } else {
            Ok(DynamicImage::ImageRgb32F(
                ImageBuffer::<Rgb<f32>, Vec<f32>>::from_raw(width, height, pixels)
                    .expect("invalid image buffer"),
            ))
        }
    }

    #[allow(deprecated)]
    fn surface_pixbuf(&self) -> Result<Pixbuf> {
        gdk::pixbuf_get_from_surface(self.0.as_ref(), 0, 0, self.0.width(), self.0.height())
            .ok_or(Error::NullPointer)
    }

    pub fn size(&self) -> Result<Size> {
        Ok(Size::new(self.0.width() as _, self.0.height() as _))
    }
}

/// An owned buffer of `f32` pixels that Cairo reads as bytes.
///
/// Cairo's `RGB96F`/`RGBA128F` surfaces interpret the data as `f32`, so the
/// allocation must be aligned for `f32`. Casting a `Vec<f32>` to a `Vec<u8>`
/// with `bytemuck::cast_vec` is not possible (the alignments differ), so the
/// buffer keeps the original `Vec<f32>` and exposes it as bytes.
struct F32Buffer(Vec<f32>);

impl F32Buffer {
    fn zeroed(bytes: usize) -> Self {
        Self(vec![0.0; bytes.div_ceil(size_of::<f32>())])
    }
}

impl AsMut<[u8]> for F32Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(&mut self.0)
    }
}

/// Premultiply the alpha channel and keep the pixels as `f32`.
fn premultiplied(mut pixels: Vec<f32>) -> F32Buffer {
    premultiply_rgba_f32(&mut pixels);
    F32Buffer(pixels)
}

fn pixbuf_to_dynamic_image(pixbuf: &Pixbuf) -> Result<DynamicImage> {
    let width = pixbuf.width() as u32;
    let height = pixbuf.height() as u32;
    let channels = pixbuf.n_channels() as usize;
    let data = packed_rows::<u8>(
        pixbuf.read_pixel_bytes().as_ref(),
        pixbuf.rowstride() as usize,
        width as usize * channels,
        height as usize,
    );
    Ok(match channels {
        3 => DynamicImage::ImageRgb8(
            RgbImage::from_raw(width, height, data).ok_or(Error::NullPointer)?,
        ),
        4 => DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, data).ok_or(Error::NullPointer)?,
        ),
        _ => return Err(Error::NotSupported),
    })
}

impl TryFrom<&DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: &DrawingImage) -> Result<Self> {
        Ok(Image::from_texture(value.to_texture()?))
    }
}

impl TryFrom<DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        Image::try_from(&value)
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
        value.to_dynamic_image()
    }
}
