use std::{
    f64::consts::{FRAC_PI_2, PI},
    io,
    rc::{Rc, Weak},
};

use gtk4::{
    cairo::Context,
    glib::object::Cast,
    pango::{FontDescription, SCALE as PANGO_SCALE, Style, Weight},
    prelude::{DrawingAreaExtManual, WidgetExt},
};
use pangocairo::functions::show_layout;

use super::callback::Callback;
use crate::{
    AsContainer, BrushPen, Container, DrawingFont, HAlign, Point, Rect, RectBox, RelativeToScreen,
    Size, SolidColorBrush, VAlign, Widget,
};

pub struct Canvas {
    widget: gtk4::DrawingArea,
    handle: Rc<Widget>,
    on_redraw: Callback<Context>,
}

impl Canvas {
    pub fn new(parent: impl AsContainer) -> io::Result<Rc<Self>> {
        let widget = gtk4::DrawingArea::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Ok(Rc::new_cyclic(|this: &Weak<Self>| {
            widget.set_draw_func({
                let this = this.clone();
                move |_, ctx, _, _| {
                    if let Some(this) = this.upgrade() {
                        this.on_redraw.signal(ctx.clone());
                    }
                }
            });
            Self {
                widget,
                handle,
                on_redraw: Callback::new(),
            }
        }))
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(self.handle.loc())
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p);
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(self.handle.size())
    }

    pub fn set_size(&self, s: Size) -> io::Result<()> {
        self.handle.set_size(s);
        Ok(())
    }

    pub fn redraw(&self) -> io::Result<()> {
        self.widget.queue_draw();
        Ok(())
    }

    pub async fn wait_redraw(&self) -> io::Result<DrawingContext> {
        let ctx = self.on_redraw.wait().await;
        Ok(DrawingContext {
            ctx,
            widget: self.widget.clone(),
        })
    }
}

impl AsContainer for Canvas {
    fn as_container(&self) -> Container {
        Container::Parent(Rc::downgrade(&self.handle))
    }
}

pub struct DrawingContext {
    ctx: Context,
    widget: gtk4::DrawingArea,
}

#[inline]
fn to_trans(rect: Rect) -> RelativeToScreen {
    RelativeToScreen::scale(rect.size.width, rect.size.height)
        .then_translate(rect.origin.to_vector())
}

impl DrawingContext {
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

    pub fn draw_arc(&self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> io::Result<()> {
        self.path_arc(rect, start, end);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
        Ok(())
    }

    pub fn fill_pie(&self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> io::Result<()> {
        self.path_arc(rect, start, end);
        self.set_brush(brush, rect);
        self.ctx.fill().ok();
        Ok(())
    }

    pub fn draw_ellipse(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        self.draw_arc(pen, rect, 0.0, PI * 2.0)
    }

    pub fn fill_ellipse(&self, brush: impl Brush, rect: Rect) -> io::Result<()> {
        self.fill_pie(brush, rect, 0.0, PI * 2.0)
    }

    pub fn draw_line(&self, pen: impl Pen, start: Point, end: Point) -> io::Result<()> {
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
        Ok(())
    }

    pub fn draw_rect(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        self.path_rect(rect);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
        Ok(())
    }

    pub fn fill_rect(&self, brush: impl Brush, rect: Rect) -> io::Result<()> {
        self.path_rect(rect);
        self.set_brush(brush, rect);
        self.ctx.fill().ok();
        Ok(())
    }

    pub fn draw_round_rect(&self, pen: impl Pen, rect: Rect, round: Size) -> io::Result<()> {
        self.path_round_rect(rect, round);
        self.set_pen(pen, rect);
        self.ctx.stroke().ok();
        Ok(())
    }

    pub fn fill_round_rect(&self, brush: impl Brush, rect: Rect, round: Size) -> io::Result<()> {
        self.path_round_rect(rect, round);
        self.set_brush(brush, rect);
        self.ctx.fill().ok();
        Ok(())
    }

    pub fn draw_str(
        &self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) -> io::Result<()> {
        let layout = self.widget.create_pango_layout(Some(text.as_ref()));
        let mut desp = FontDescription::from_string(&font.family);
        desp.set_size(font.size as i32 * PANGO_SCALE);
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
        Ok(())
    }
}

pub trait Brush {
    fn set(&self, ctx: &Context, trans: RelativeToScreen);
}

impl<B: Brush> Brush for &'_ B {
    fn set(&self, ctx: &Context, trans: RelativeToScreen) {
        (**self).set(ctx, trans)
    }
}

impl Brush for SolidColorBrush {
    fn set(&self, ctx: &Context, _trans: RelativeToScreen) {
        ctx.set_source_rgba(
            self.color.r as f64 / 255.0,
            self.color.g as f64 / 255.0,
            self.color.b as f64 / 255.0,
            self.color.a as f64 / 255.0,
        );
    }
}

pub trait Pen {
    fn set(&self, ctx: &Context, trans: RelativeToScreen);
}

impl<P: Pen> Pen for &'_ P {
    fn set(&self, ctx: &Context, trans: RelativeToScreen) {
        (**self).set(ctx, trans)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn set(&self, ctx: &Context, trans: RelativeToScreen) {
        self.brush.set(ctx, trans);
        ctx.set_line_width(self.width);
    }
}
