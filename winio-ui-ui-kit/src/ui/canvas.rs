use std::{
    cell::{Cell, RefCell},
    ptr::null_mut,
};

use compio_log::*;
use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_core_foundation::{CFRange, CFRetained, CGAffineTransform, CGPoint};
use objc2_core_graphics::{
    CGAffineTransformMake, CGAffineTransformMakeScale, CGColor, CGContext, CGMutablePath, CGPath,
    kCGColorWhite,
};
use objc2_core_text::CTFramesetter;
use objc2_foundation::{MainThreadMarker, NSRect, NSSet, NSSize};
use objc2_ui_kit::{UIEvent, UIGraphicsGetCurrentContext, UITouch, UIView};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{
    ColorTheme, DrawingFont, HAlign, MouseButton, Point, Rect, Size, Transform, VAlign, Vector,
};

use crate::{
    Brush, DrawAction, DrawingImage, Error, GlobalRuntime, Pen, Result, Widget, catch,
    create_attr_str, from_cgsize, to_cgpoint, to_cgrect, transform_cgpoint, transform_rect,
};

#[derive(Debug)]
pub struct Canvas {
    view: Retained<CanvasView>,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let view = catch(|| {
            let view = CanvasView::new(parent.as_ui_kit().mtm());
            view.setTransform(CGAffineTransformMakeScale(1.0, -1.0));
            view
        })?;
        let handle = Widget::from_uiview(parent, view.clone().into_super())?;
        Ok(Self { view, handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        Ok(DrawingContext {
            size: self.size()?,
            actions: self.view.ivars().take_buffer(),
            canvas: self,
            transform: Transform::identity(),
            ended: false,
        })
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.view.ivars().touches_began.wait().await;
        MouseButton::Left
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.view.ivars().touches_ended.wait().await;
        MouseButton::Left
    }

    pub async fn wait_mouse_move(&self) -> Point {
        let p = self.view.ivars().touches_moved.wait().await;
        let size = self.view.frame().size;
        let size = from_cgsize(size);
        transform_cgpoint(size, p)
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

fn draw_rect(actions: &[DrawAction], rect: NSRect, factor: f64) {
    let Some(context) = UIGraphicsGetCurrentContext() else {
        error!("Cannot get current CGContext");
        return;
    };
    if !matches!(crate::color_theme(), Ok(ColorTheme::Dark)) {
        CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, 1.0);
        CGContext::fill_rect(Some(&context), rect);
    } else {
        CGContext::clear_rect(Some(&context), rect);
    }
    DrawAction::draw_rect(actions, &context, factor);
}

#[derive(Debug, Default)]
struct CanvasViewIvars {
    touches_began: Callback,
    touches_moved: Callback<CGPoint>,
    touches_ended: Callback,
    actions: RefCell<Vec<DrawAction>>,
    actions_buf: RefCell<Vec<DrawAction>>,
    factor: Cell<f64>,
}

impl CanvasViewIvars {
    pub fn take_buffer(&self) -> Vec<DrawAction> {
        std::mem::take(&mut self.actions_buf.borrow_mut())
    }

    pub fn swap_buffer(&self, buf: &mut Vec<DrawAction>) {
        {
            let mut actions = self.actions.borrow_mut();
            std::mem::swap::<Vec<DrawAction>>(&mut actions, buf);
        }
        {
            let mut actions_buf = self.actions_buf.borrow_mut();
            std::mem::swap::<Vec<DrawAction>>(&mut actions_buf, buf);
            actions_buf.clear();
        }
    }
}

define_class! {
    #[unsafe(super(UIView))]
    #[name = "WinioCanvasViewUIKit"]
    #[ivars = CanvasViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct CanvasView;

    #[allow(non_snake_case)]
    impl CanvasView {
        #[unsafe(method_id(initWithFrame:))]
        fn initWithFrame(this: Allocated<Self>, frame: NSRect) -> Option<Retained<Self>> {
            let this = this.set_ivars(CanvasViewIvars::default());
            unsafe { msg_send![super(this), initWithFrame: frame] }
        }

        #[unsafe(method(drawRect:))]
        unsafe fn drawRect(&self, rect: NSRect) {
            let ivars = self.ivars();
            draw_rect(&ivars.actions.borrow(), rect, ivars.factor.get())
        }

        #[unsafe(method(touchesBegan:withEvent:))]
        unsafe fn touchesBegan(&self, _touches: &NSSet<UITouch>, _event: &UIEvent) {
            self.ivars().touches_began.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(touchesMoved:withEvent:))]
        unsafe fn touchesMoved(&self, touches: &NSSet<UITouch>, _event: &UIEvent) {
            if let Some(touch) = touches.iter().next() {
                self.ivars().touches_moved.signal::<GlobalRuntime>(touch.locationInView(Some(self)));
            }
        }

        #[unsafe(method(touchesEnded:withEvent:))]
        unsafe fn touchesEnded(&self, _touches: &NSSet<UITouch>, _event: &UIEvent) {
            self.ivars().touches_ended.signal::<GlobalRuntime>(());
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), initWithFrame: NSRect::ZERO] }
    }
}

pub struct DrawingContext<'a> {
    size: Size,
    actions: Vec<DrawAction>,
    canvas: &'a mut Canvas,
    transform: Transform,
    ended: bool,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let Err(_e) = self.end() {
            error!("Error dropping DrawingContext: {_e:?}");
        }
    }
}

impl DrawingContext<'_> {
    fn end(&mut self) -> Result<()> {
        if !self.ended {
            let ivars = self.canvas.view.ivars();
            ivars.swap_buffer(&mut self.actions);
            ivars.factor.set(
                self.canvas
                    .view
                    .window()
                    .map(|w| w.screen().scale())
                    .unwrap_or(1.0),
            );
            catch(|| self.canvas.view.setNeedsDisplay())?;
            self.ended = true;
        }
        Ok(())
    }

    pub fn close(mut self) -> Result<()> {
        self.end()
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<()> {
        self.transform = transform;
        self.actions.push(DrawAction::Transform(CGAffineTransform {
            a: transform.m11,
            b: transform.m12,
            c: transform.m21,
            d: transform.m22,
            tx: transform.m31,
            ty: transform.m32,
        }));
        Ok(())
    }

    pub fn transform(&self) -> Result<Transform> {
        Ok(self.transform)
    }

    fn draw(&mut self, pen: impl Pen, path: CFRetained<CGPath>) -> Result<()> {
        self.actions.push(pen.create_action(path)?);
        Ok(())
    }

    fn fill(&mut self, brush: impl Brush, path: CFRetained<CGPath>) -> Result<()> {
        self.actions.push(brush.create_action(path)?);
        Ok(())
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) -> Result<()> {
        self.draw(pen, path.0.clone())
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) -> Result<()> {
        self.fill(brush, path.0.clone())
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let path = path_arc(self.size, rect, start, end, false);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let path = path_arc(self.size, rect, start, end, true);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> Result<()> {
        let path = path_arc(self.size, rect, start, end, true);
        self.fill(brush, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let path = path_ellipse(self.size, rect);
        self.draw(pen, path)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let path = path_ellipse(self.size, rect);
        self.fill(brush, path)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) -> Result<()> {
        let path = path_line(self.size, start, end);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let path = path_rect(self.size, rect);
        self.draw(pen, path)
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let path = path_rect(self.size, rect);
        self.fill(brush, path)
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) -> Result<()> {
        let path = path_round_rect(self.size, rect, round);
        self.draw(pen, path)
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) -> Result<()> {
        let path = path_round_rect(self.size, rect, round);
        self.fill(brush, path)
    }

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: &str,
    ) -> Result<()> {
        let color = brush.text_color()?;
        let (framesetter, rect) = measure_str(font, &color, pos, text, self.size)?;
        let rect = to_cgrect(rect);
        self.actions
            .push(brush.create_text_action(framesetter, rect)?);
        Ok(())
    }

    pub fn measure_str(&self, font: DrawingFont, text: &str) -> Result<Size> {
        let color =
            unsafe { CGColor::constant_color(Some(kCGColorWhite)).ok_or(Error::NullPointer) }?;
        Ok(measure_str(font, &color, Point::zero(), text, self.size)?
            .1
            .size)
    }

    pub fn create_image(&self, image: DynamicImage) -> Result<DrawingImage> {
        DrawingImage::new(image)
    }

    pub fn draw_image(
        &mut self,
        image_rep: &DrawingImage,
        rect: Rect,
        clip: Option<Rect>,
    ) -> Result<()> {
        let rect = transform_rect(self.size, rect);
        let clip = clip.map(to_cgrect);
        self.actions
            .push(DrawAction::Image(image_rep.clone(), rect, clip));
        Ok(())
    }

    pub fn create_path_builder(&self, start: Point) -> Result<DrawingPathBuilder> {
        Ok(DrawingPathBuilder::new(self.size, start))
    }
}

pub struct DrawingPath(CFRetained<CGPath>);

pub struct DrawingPathBuilder {
    size: Size,
    matrix: CGAffineTransform,
    path: CFRetained<CGMutablePath>,
}

impl DrawingPathBuilder {
    fn new(size: Size, start: Point) -> Self {
        unsafe {
            let path = CGMutablePath::new();
            let matrix = flip_transform(size);
            let p = to_cgpoint(start);
            CGMutablePath::move_to_point(Some(&path), &matrix, p.x, p.y);
            Self { size, matrix, path }
        }
    }

    pub fn add_line(&mut self, p: Point) -> Result<()> {
        let p = to_cgpoint(p);
        unsafe {
            CGMutablePath::add_line_to_point(Some(&self.path), &self.matrix, p.x, p.y);
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
        let startp = Point::new(
            center.x + radius.width * start.cos(),
            center.y + radius.height * start.sin(),
        );

        let rate = radius.height / radius.width;
        let transform = CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0);

        self.add_line(startp)?;
        let mut center = to_cgpoint(center);
        center.y = self.size.height - center.y;
        unsafe {
            CGMutablePath::add_arc(
                Some(&self.path),
                &transform,
                center.x,
                center.y / rate,
                radius.width,
                -start,
                -end,
                clockwise,
            );
        }
        Ok(())
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) -> Result<()> {
        let p1 = to_cgpoint(p1);
        let p2 = to_cgpoint(p2);
        let p3 = to_cgpoint(p3);
        unsafe {
            CGMutablePath::add_curve_to_point(
                Some(&self.path),
                &self.matrix,
                p1.x,
                p1.y,
                p2.x,
                p2.y,
                p3.x,
                p3.y,
            );
        }
        Ok(())
    }

    pub fn build(self, close: bool) -> Result<DrawingPath> {
        unsafe {
            if close {
                CGMutablePath::close_subpath(Some(&self.path));
            }
            Ok(DrawingPath(CFRetained::cast_unchecked(self.path)))
        }
    }
}

fn flip_transform(s: Size) -> CGAffineTransform {
    CGAffineTransformMake(1.0, 0.0, 0.0, -1.0, 0.0, s.height)
}

fn path_arc(s: Size, rect: Rect, start: f64, end: f64, pie: bool) -> CFRetained<CGMutablePath> {
    let radius = rect.size / 2.0;
    let centerp = Point::new(rect.origin.x + radius.width, rect.origin.y + radius.height);
    let startp = Point::new(
        centerp.x + radius.width * start.cos(),
        centerp.y + radius.height * start.sin(),
    );

    let rate = radius.height / radius.width;
    let transform = CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0);
    let trivial_transform = flip_transform(s);

    unsafe {
        let path = CGMutablePath::new();
        let centerp = to_cgpoint(centerp);
        let startp = to_cgpoint(startp);
        if pie {
            CGMutablePath::move_to_point(Some(&path), &trivial_transform, centerp.x, centerp.y);
            CGMutablePath::add_line_to_point(
                Some(&path),
                &trivial_transform,
                startp.x,
                startp.y / rate,
            );
        } else {
            CGMutablePath::move_to_point(Some(&path), &trivial_transform, startp.x, startp.y);
        }
        CGMutablePath::add_arc(
            Some(&path),
            &transform,
            centerp.x,
            centerp.y / rate,
            radius.width,
            -start,
            -end,
            true,
        );
        if pie {
            CGMutablePath::close_subpath(Some(&path));
        }
        path
    }
}

fn path_ellipse(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = to_cgrect(rect);
    let transform = flip_transform(s);
    unsafe { CGPath::with_ellipse_in_rect(rect, &transform) }
}

fn path_line(s: Size, start: Point, end: Point) -> CFRetained<CGMutablePath> {
    unsafe {
        let path = CGMutablePath::new();
        let transform = flip_transform(s);
        let p = to_cgpoint(start);
        CGMutablePath::move_to_point(Some(&path), &transform, p.x, p.y);
        let p = to_cgpoint(end);
        CGMutablePath::add_line_to_point(Some(&path), &transform, p.x, p.y);
        path
    }
}

fn path_rect(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = to_cgrect(rect);
    let transform = flip_transform(s);
    unsafe { CGPath::with_rect(rect, &transform) }
}

fn path_round_rect(s: Size, rect: Rect, round: Size) -> CFRetained<CGPath> {
    let rect = to_cgrect(rect);
    let transform = flip_transform(s);
    unsafe { CGPath::with_rounded_rect(rect, round.width, round.height, &transform) }
}

fn measure_str(
    font: DrawingFont,
    color: &CGColor,
    pos: Point,
    text: &str,
    bound: Size,
) -> Result<(CFRetained<CTFramesetter>, Rect)> {
    let astr = create_attr_str(&font, color, text)?;
    let framesetter = unsafe { CTFramesetter::with_attributed_string(&astr) };
    let size = from_cgsize(unsafe {
        framesetter.suggest_frame_size_with_constraints(
            CFRange::new(0, 0),
            None,
            NSSize::new(bound.width, bound.height + font.size),
            null_mut(),
        )
    });
    let mut x = pos.x;
    let mut y = bound.height - pos.y;
    match font.halign {
        HAlign::Center => x -= size.width / 2.0,
        HAlign::Right => x -= size.width,
        _ => {}
    }
    match font.valign {
        VAlign::Center => y -= size.height / 2.0,
        VAlign::Top => y -= size.height,
        _ => {}
    }
    Ok((framesetter, Rect::new(Point::new(x, y), size)))
}
