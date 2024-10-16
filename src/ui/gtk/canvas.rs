use std::{
    cell::RefCell,
    f64::consts::{FRAC_PI_2, PI},
    rc::Rc,
};

use gtk4::{
    EventControllerMotion, GestureClick,
    cairo::{
        Content, Context, Format, ImageSurface, LinearGradient, Matrix, RadialGradient,
        RecordingSurface,
    },
    glib::object::Cast,
    pango::{FontDescription, SCALE as PANGO_SCALE, Style, Weight},
    prelude::{DrawingAreaExtManual, GestureSingleExt, WidgetExt},
};
use image::DynamicImage;
use pangocairo::functions::show_layout;

use crate::{
    AsWindow, BrushPen, DrawingFont, HAlign, LinearGradientBrush, MouseButton, Point,
    RadialGradientBrush, Rect, RectBox, RelativeToLogical, Size, SolidColorBrush, VAlign,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct Canvas {
    on_motion: Rc<Callback<Point>>,
    on_pressed: Rc<Callback<MouseButton>>,
    on_released: Rc<Callback<MouseButton>>,
    widget: gtk4::DrawingArea,
    handle: Widget,
    surface: Rc<RefCell<RecordingSurface>>,
}

impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::DrawingArea::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });

        let on_motion = Rc::new(Callback::new());
        let on_pressed = Rc::new(Callback::new());
        let on_released = Rc::new(Callback::new());

        let surface = Rc::new(RefCell::new(
            RecordingSurface::create(Content::ColorAlpha, None).unwrap(),
        ));

        widget.set_draw_func({
            let surface = surface.clone();
            move |_, ctx, _, _| {
                ctx.set_source_surface(&*surface.borrow(), 0.0, 0.0)
                    .unwrap();
                ctx.paint().unwrap();
            }
        });

        let controller = EventControllerMotion::new();
        controller.connect_motion({
            let on_motion = Rc::downgrade(&on_motion);
            move |_, x, y| {
                if let Some(on_motion) = on_motion.upgrade() {
                    on_motion.signal(Point::new(x, y));
                }
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
            let on_pressed = Rc::downgrade(&on_pressed);
            move |controller, _, _, _| {
                if let Some(on_pressed) = on_pressed.upgrade() {
                    on_pressed.signal(gtk_current_button(controller.current_button()));
                }
            }
        });
        controller.connect_released({
            let on_released = Rc::downgrade(&on_released);
            move |controller, _, _, _| {
                if let Some(on_released) = on_released.upgrade() {
                    on_released.signal(gtk_current_button(controller.current_button()));
                }
            }
        });
        widget.add_controller(controller);

        Self {
            on_motion,
            on_pressed,
            on_released,
            widget,
            handle,
            surface,
        }
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, s: Size) {
        self.handle.set_size(s);
    }

    pub fn context(&mut self) -> DrawingContext<'_> {
        let mut surface = self.surface.borrow_mut();
        *surface = RecordingSurface::create(Content::ColorAlpha, None).unwrap();
        DrawingContext {
            ctx: Context::new(&*surface).unwrap(),
            widget: &mut self.widget,
        }
    }

    pub async fn wait_redraw(&self) {
        std::future::pending().await
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
}

pub struct DrawingContext<'a> {
    ctx: Context,
    widget: &'a mut gtk4::DrawingArea,
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
    #[inline]
    fn set_brush(&self, brush: impl Brush, rect: Rect) {
        brush.set(&self.ctx, to_trans(rect))
    }

    #[inline]
    fn set_pen(&self, pen: impl Pen, rect: Rect) {
        pen.set(&self.ctx, to_trans(rect))
    }

    fn path_arc(&self, rect: Rect, start: f64, end: f64) {
        let save_matrix = self.ctx.matrix();
        let rate = rect.size.height / rect.size.width;
        self.ctx.scale(1.0, rate);
        self.ctx.new_path();
        let center = rect.center();
        self.ctx
            .arc(center.x, center.y / rate, rect.size.width / 2.0, start, end);
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

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        self.path_arc(rect, start, end);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        self.path_arc(rect, start, end);
        self.set_brush(brush, rect);
        self.ctx.fill().ok();
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        self.draw_arc(pen, rect, 0.0, PI * 2.0)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        self.fill_pie(brush, rect, 0.0, PI * 2.0)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        self.ctx.new_path();
        self.ctx.move_to(start.x, start.y);
        self.ctx.line_to(end.x, end.y);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        self.path_rect(rect);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        self.path_rect(rect);
        self.set_brush(brush, rect);
        self.ctx.fill().ok();
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        self.path_round_rect(rect, round);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        self.path_round_rect(rect, round);
        self.set_brush(brush, rect);
        self.ctx.fill().ok();
    }

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str) {
        let layout = self.widget.create_pango_layout(Some(text));
        let mut desp = FontDescription::from_string(&font.family);
        desp.set_size((font.size / 1.33) as i32 * PANGO_SCALE);
        if font.italic {
            desp.set_style(Style::Italic);
        }
        if font.bold {
            desp.set_weight(Weight::Bold);
        }
        layout.set_font_description(Some(&desp));

        let (width, height) = layout.pixel_size();
        let (width, height) = (width as f64, height as f64);

        let mut x = pos.x;
        let mut y = pos.y;
        match font.halign {
            HAlign::Center => x -= width / 2.0,
            HAlign::Right => x -= width,
            _ => {}
        }
        match font.valign {
            VAlign::Center => y -= height / 2.0,
            VAlign::Bottom => y -= height,
            _ => {}
        }
        let rect = Rect::new(Point::new(x, y), Size::new(width, height));

        self.ctx.move_to(rect.origin.x, rect.origin.y);
        self.set_brush(brush, rect);
        show_layout(&self.ctx, &layout);
    }

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage {
        DrawingImage::new(image)
    }

    pub fn draw_image(&mut self, image: &DrawingImage, rect: Rect, clip: Option<Rect>) {
        self.ctx.save().unwrap();
        let clip = clip.unwrap_or_else(|| Rect::new(Point::zero(), image.size()));
        self.ctx.rectangle(
            clip.origin.x,
            clip.origin.y,
            clip.size.width,
            clip.size.height,
        );
        self.ctx.clip();
        self.ctx.new_path();
        let size = image.size();
        self.ctx.translate(rect.origin.x, rect.origin.y);
        self.ctx
            .scale(rect.width() / size.width, rect.height() / size.height);
        self.ctx
            .set_source_surface(&image.0, -clip.origin.x, -clip.origin.y)
            .unwrap();
        self.ctx.paint().unwrap();
        self.ctx.restore().unwrap();
    }
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        self.widget.queue_draw();
    }
}

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn set(&self, ctx: &Context, trans: RelativeToLogical);
}

impl<B: Brush> Brush for &'_ B {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) {
        (**self).set(ctx, trans)
    }
}

impl Brush for SolidColorBrush {
    fn set(&self, ctx: &Context, _trans: RelativeToLogical) {
        ctx.set_source_rgba(
            self.color.r as f64 / 255.0,
            self.color.g as f64 / 255.0,
            self.color.b as f64 / 255.0,
            self.color.a as f64 / 255.0,
        );
    }
}

impl Brush for LinearGradientBrush {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) {
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
        ctx.set_source(&p).unwrap();
    }
}

impl Brush for RadialGradientBrush {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) {
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
        ctx.set_source(&p).unwrap();
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn set(&self, ctx: &Context, trans: RelativeToLogical);
}

impl<P: Pen> Pen for &'_ P {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) {
        (**self).set(ctx, trans)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn set(&self, ctx: &Context, trans: RelativeToLogical) {
        self.brush.set(ctx, trans);
        ctx.set_line_width(self.width);
    }
}

pub struct DrawingImage(ImageSurface);

impl DrawingImage {
    fn new(image: DynamicImage) -> Self {
        let width = image.width();
        let height = image.height();
        let (format, buffer) = match image {
            DynamicImage::ImageRgb32F(_) => (Format::__Unknown(6), image.into_bytes()), /* CAIRO_FORMAT_RGB96F */
            DynamicImage::ImageRgba32F(_) => (Format::__Unknown(7), image.into_bytes()), /* CAIRO_FORMAT_RGBA128F */
            _ => (
                Format::__Unknown(7),
                DynamicImage::ImageRgba32F(image.into_rgba32f()).into_bytes(),
            ),
        };
        let stride = format.stride_for_width(width).unwrap();
        let surface =
            ImageSurface::create_for_data(buffer, format, width as _, height as _, stride as _)
                .unwrap();
        Self(surface)
    }

    pub fn size(&self) -> Size {
        Size::new(self.0.width() as _, self.0.height() as _)
    }
}
