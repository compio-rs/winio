use std::{
    cell::RefCell,
    f64::consts::{FRAC_PI_2, PI},
    rc::Rc,
};

use gtk4::{
    EventControllerMotion, GestureClick,
    cairo::{Content, Context, RecordingSurface},
    glib::object::Cast,
    pango::{FontDescription, SCALE as PANGO_SCALE, Style, Weight},
    prelude::{DrawingAreaExtManual, GestureSingleExt, WidgetExt},
};
use pangocairo::functions::show_layout;

use crate::{
    AsWindow, BrushPen, DrawingFont, HAlign, MouseButton, Point, Rect, RectBox, RelativeToScreen,
    Size, SolidColorBrush, VAlign,
    ui::{Callback, Widget},
};

pub struct Canvas {
    on_redraw: Rc<Callback<()>>,
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

        let on_redraw = Rc::new(Callback::new());
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
            on_redraw,
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

    pub fn redraw(&self) {
        self.on_redraw.signal(());
    }

    pub async fn wait_redraw(&self) {
        self.on_redraw.wait().await;
    }

    pub fn context(&mut self) -> DrawingContext<'_> {
        let mut surface = self.surface.borrow_mut();
        *surface = RecordingSurface::create(Content::ColorAlpha, None).unwrap();
        DrawingContext {
            ctx: Context::new(&*surface).unwrap(),
            widget: &mut self.widget,
        }
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
fn to_trans(rect: Rect) -> RelativeToScreen {
    RelativeToScreen::scale(rect.size.width, rect.size.height)
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

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) {
        let layout = self.widget.create_pango_layout(Some(text.as_ref()));
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
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        self.widget.queue_draw();
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
