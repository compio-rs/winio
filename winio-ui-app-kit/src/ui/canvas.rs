use std::{
    cell::{Cell, RefCell},
    ptr::{null, null_mut},
    rc::Rc,
};

use compio_log::*;
use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use objc2::{
    AllocAnyThread, DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{
    NSBitmapFormat, NSBitmapImageRep, NSDeviceRGBColorSpace, NSEvent, NSEventType,
    NSGraphicsContext, NSView,
};
use objc2_core_foundation::{
    CFMutableArray, CFMutableAttributedString, CFRange, CFRetained, kCFAllocatorDefault,
};
use objc2_core_graphics::{
    CGAffineTransformMake, CGBitmapContextCreate, CGBitmapContextCreateImage, CGColor,
    CGColorSpace, CGContext, CGGradient, CGGradientDrawingOptions, CGMutablePath, CGPath,
    kCGColorWhite,
};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontSymbolicTraits, CTFramesetter, kCTFontAttributeName,
    kCTForegroundColorAttributeName,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{
    BrushPen, Color, DrawingFont, GradientStop, HAlign, LinearGradientBrush, MouseButton, Point,
    RadialGradientBrush, Rect, RelativePoint, Size, SolidColorBrush, VAlign,
};

use crate::{
    GlobalRuntime,
    ui::{TollFreeBridge, Widget, from_cgsize, transform_cgpoint, transform_point, transform_rect},
};

#[derive(Debug)]
pub struct Canvas {
    view: Retained<CanvasView>,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let view = CanvasView::new(MainThreadMarker::new().unwrap());
        let handle = Widget::from_nsview(parent, view.clone().into_super());
        Self { view, handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn context(&mut self) -> DrawingContext<'_> {
        DrawingContext {
            size: self.size(),
            actions: self.view.ivars().take_buffer(),
            canvas: self,
        }
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.view.ivars().mouse_down.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.view.ivars().mouse_up.wait().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.view.ivars().mouse_move.wait().await;
        self.view
            .window()
            .map(|w| {
                let p = unsafe { w.mouseLocationOutsideOfEventStream() };
                transform_cgpoint(self.size(), p)
            })
            .unwrap()
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub enum DrawGradientAction {
    Linear {
        gradient: CFRetained<CGGradient>,
        start_point: NSPoint,
        end_point: NSPoint,
    },
    Radial {
        gradient: CFRetained<CGGradient>,
        start_center: NSPoint,
        start_radius: f64,
        end_center: NSPoint,
        end_radius: f64,
    },
}

impl DrawGradientAction {
    unsafe fn draw(&self, context: &CGContext) {
        match self {
            Self::Linear {
                gradient,
                start_point,
                end_point,
            } => {
                CGContext::draw_linear_gradient(
                    Some(context),
                    Some(gradient),
                    *start_point,
                    *end_point,
                    CGGradientDrawingOptions::all(),
                );
            }
            Self::Radial {
                gradient,
                start_center,
                start_radius,
                end_center,
                end_radius,
            } => {
                CGContext::draw_radial_gradient(
                    Some(context),
                    Some(gradient),
                    *start_center,
                    *start_radius,
                    *end_center,
                    *end_radius,
                    CGGradientDrawingOptions::all(),
                );
            }
        }
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub enum DrawAction {
    Path(CFRetained<CGPath>, CFRetained<CGColor>, Option<f64>),
    GradientPath(CFRetained<CGPath>, DrawGradientAction, Option<f64>),
    Text(CFRetained<CTFramesetter>, NSRect),
    GradientText(CFRetained<CTFramesetter>, DrawGradientAction, NSRect),
    Image(DrawingImage, NSRect, Option<NSRect>),
}

impl DrawAction {
    fn with_width(self, width: f64) -> Self {
        match self {
            Self::Path(path, color, _) => Self::Path(path, color, Some(width)),
            Self::GradientPath(path, gradient, _) => {
                Self::GradientPath(path, gradient, Some(width))
            }
            _ => self,
        }
    }

    unsafe fn draw_rect(actions: &[Self], _rect: NSRect, factor: f64) {
        let Some(ns_context) = NSGraphicsContext::currentContext() else {
            error!("Cannot get current NSGraphicsContext");
            return;
        };
        let context = ns_context.CGContext();
        for action in actions {
            CGContext::save_g_state(Some(&context));
            match action {
                Self::Path(path, color, width) => {
                    CGContext::add_path(Some(&context), Some(path));
                    if let Some(width) = width {
                        CGContext::set_stroke_color_with_color(Some(&context), Some(color));
                        CGContext::set_line_width(Some(&context), *width);
                        CGContext::stroke_path(Some(&context));
                    } else {
                        CGContext::set_fill_color_with_color(Some(&context), Some(color));
                        CGContext::fill_path(Some(&context));
                    }
                }
                Self::GradientPath(path, gradient, width) => {
                    CGContext::add_path(Some(&context), Some(path));
                    if let Some(width) = width {
                        CGContext::set_line_width(Some(&context), *width);
                        CGContext::replace_path_with_stroked_path(Some(&context));
                        CGContext::clip(Some(&context));
                        gradient.draw(&context);
                    } else {
                        CGContext::clip(Some(&context));
                        gradient.draw(&context);
                    }
                }
                Self::Text(framesetter, rect) => {
                    let text_path = CGPath::with_rect(*rect, null());

                    let frame = framesetter.frame(CFRange::new(0, 0), &text_path, None);

                    frame.draw(&context);
                }
                Self::GradientText(framesetter, gradient, rect) => {
                    let colorspace = CGColorSpace::new_device_gray();
                    let Some(mask) = CGBitmapContextCreate(
                        null_mut(),
                        (rect.size.width * factor) as _,
                        (rect.size.height * factor) as _,
                        8,
                        (rect.size.width * factor) as _,
                        colorspace.as_deref(),
                        0,
                    ) else {
                        error!("Cannot create CGBitmapContext");
                        continue;
                    };

                    let text_path =
                        CGPath::with_rect(NSRect::new(NSPoint::ZERO, rect.size), null());

                    let frame = framesetter.frame(CFRange::new(0, 0), &text_path, None);

                    CGContext::scale_ctm(Some(&mask), factor, factor);
                    frame.draw(&mask);

                    let mask_image = CGBitmapContextCreateImage(Some(&mask));
                    CGContext::clip_to_mask(Some(&context), *rect, mask_image.as_deref());
                    gradient.draw(&context);
                }
                Self::Image(image, rect, clip) => {
                    let cg_image = image.rep.CGImage();
                    if let Some(clip) = clip {
                        CGContext::clip_to_rect(Some(&context), *clip);
                    }
                    CGContext::draw_image(Some(&context), *rect, cg_image.as_deref());
                }
            }
            CGContext::restore_g_state(Some(&context));
        }
    }
}

#[derive(Debug, Default)]
struct CanvasViewIvars {
    mouse_down: Callback<MouseButton>,
    mouse_up: Callback<MouseButton>,
    mouse_move: Callback,
    actions: RefCell<Vec<DrawAction>>,
    // A buffer for actions, to avoid frequent allocations.
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
    #[unsafe(super(NSView))]
    #[name = "WinioCanvasView"]
    #[ivars = CanvasViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct CanvasView;

    #[allow(non_snake_case)]
    impl CanvasView {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(CanvasViewIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(acceptsFirstResponder))]
        unsafe fn acceptsFirstResponder(&self) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        unsafe fn drawRect(&self, rect: NSRect) {
            let ivars = self.ivars();
            DrawAction::draw_rect(&ivars.actions.borrow(), rect, ivars.factor.get())
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouseDown(&self, event: &NSEvent) {
            self.ivars().mouse_down.signal::<GlobalRuntime>(mouse_button(event));
        }

        #[unsafe(method(mouseUp:))]
        unsafe fn mouseUp(&self, event: &NSEvent) {
            self.ivars().mouse_up.signal::<GlobalRuntime>(mouse_button(event));
        }

        #[unsafe(method(mouseDragged:))]
        unsafe fn mouseDragged(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(mouseMoved:))]
        unsafe fn mouseMoved(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal::<GlobalRuntime>(());
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

unsafe fn mouse_button(event: &NSEvent) -> MouseButton {
    match event.r#type() {
        NSEventType::LeftMouseDown | NSEventType::LeftMouseUp => MouseButton::Left,
        NSEventType::RightMouseDown | NSEventType::RightMouseUp => MouseButton::Right,
        _ => MouseButton::Other,
    }
}

pub struct DrawingContext<'a> {
    size: Size,
    actions: Vec<DrawAction>,
    canvas: &'a mut Canvas,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        let ivars = self.canvas.view.ivars();
        ivars.swap_buffer(&mut self.actions);
        ivars.factor.set(
            self.canvas
                .view
                .window()
                .map(|w| w.backingScaleFactor())
                .unwrap_or(1.0),
        );
        unsafe {
            self.canvas.view.setNeedsDisplay(true);
        }
    }
}

impl DrawingContext<'_> {
    fn draw(&mut self, pen: impl Pen, path: CFRetained<CGPath>) {
        self.actions.push(pen.create_action(path));
    }

    fn fill(&mut self, brush: impl Brush, path: CFRetained<CGPath>) {
        self.actions.push(brush.create_action(path));
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) {
        self.draw(pen, path.0.clone());
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) {
        self.fill(brush, path.0.clone())
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, false);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, true);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, true);
        self.fill(brush, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        let path = path_ellipse(self.size, rect);
        self.draw(pen, path)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        let path = path_ellipse(self.size, rect);
        self.fill(brush, path)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        let path = path_line(self.size, start, end);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        let path = path_rect(self.size, rect);
        self.draw(pen, path)
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        let path = path_rect(self.size, rect);
        self.fill(brush, path)
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        let path = path_round_rect(self.size, rect, round);
        self.draw(pen, path)
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        let path = path_round_rect(self.size, rect, round);
        self.fill(brush, path)
    }

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str) {
        let (framesetter, rect) = measure_str(font, &brush.text_color(), pos, text, self.size);
        let rect = transform_rect(self.size, rect);
        self.actions
            .push(brush.create_text_action(framesetter, rect))
    }

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage {
        DrawingImage::new(image)
    }

    pub fn draw_image(&mut self, image_rep: &DrawingImage, rect: Rect, clip: Option<Rect>) {
        let rect = transform_rect(self.size, rect);
        let clip = clip.map(|clip| transform_rect(self.size, clip));
        self.actions
            .push(DrawAction::Image(image_rep.clone(), rect, clip))
    }

    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder {
        DrawingPathBuilder::new(self.size, start)
    }
}

pub struct DrawingPath(CFRetained<CGPath>);

pub struct DrawingPathBuilder {
    size: Size,
    path: CFRetained<CGMutablePath>,
}

impl DrawingPathBuilder {
    fn new(size: Size, start: Point) -> Self {
        unsafe {
            let path = CGMutablePath::new();
            let p = transform_point(size, start);
            CGMutablePath::move_to_point(Some(&path), null(), p.x, p.y);
            Self { size, path }
        }
    }

    pub fn add_line(&mut self, p: Point) {
        let p = transform_point(self.size, p);
        unsafe {
            CGMutablePath::add_line_to_point(Some(&self.path), null(), p.x, p.y);
        }
    }

    pub fn add_arc(&mut self, center: Point, radius: Size, start: f64, end: f64, clockwise: bool) {
        let startp = Point::new(
            center.x + radius.width * start.cos(),
            center.y + radius.height * start.sin(),
        );

        let rate = radius.height / radius.width;
        let transform = unsafe { CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0) };

        self.add_line(startp);
        let center = transform_point(self.size, center);
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
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) {
        let p1 = transform_point(self.size, p1);
        let p2 = transform_point(self.size, p2);
        let p3 = transform_point(self.size, p3);
        unsafe {
            CGMutablePath::add_curve_to_point(
                Some(&self.path),
                null(),
                p1.x,
                p1.y,
                p2.x,
                p2.y,
                p3.x,
                p3.y,
            );
        }
    }

    pub fn build(self, close: bool) -> DrawingPath {
        unsafe {
            if close {
                CGMutablePath::close_subpath(Some(&self.path));
            }
            DrawingPath(CFRetained::cast_unchecked(self.path))
        }
    }
}

fn path_arc(s: Size, rect: Rect, start: f64, end: f64, pie: bool) -> CFRetained<CGMutablePath> {
    let radius = rect.size / 2.0;
    let centerp = Point::new(rect.origin.x + radius.width, rect.origin.y + radius.height);
    let startp = Point::new(
        centerp.x + radius.width * start.cos(),
        centerp.y + radius.height * start.sin(),
    );

    let rate = radius.height / radius.width;
    let transform = unsafe { CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0) };

    unsafe {
        let path = CGMutablePath::new();
        let centerp = transform_point(s, centerp);
        let startp = transform_point(s, startp);
        if pie {
            CGMutablePath::move_to_point(Some(&path), null(), centerp.x, centerp.y);
            CGMutablePath::add_line_to_point(Some(&path), null(), startp.x, startp.y / rate);
        } else {
            CGMutablePath::move_to_point(Some(&path), null(), startp.x, startp.y);
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
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_ellipse_in_rect(rect, null()) }
}

fn path_line(s: Size, start: Point, end: Point) -> CFRetained<CGMutablePath> {
    unsafe {
        let path = CGMutablePath::new();
        let p = transform_point(s, start);
        CGMutablePath::move_to_point(Some(&path), null(), p.x, p.y);
        let p = transform_point(s, end);
        CGMutablePath::add_line_to_point(Some(&path), null(), p.x, p.y);
        path
    }
}

fn path_rect(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_rect(rect, null()) }
}

fn path_round_rect(s: Size, rect: Rect, round: Size) -> CFRetained<CGPath> {
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_rounded_rect(rect, round.width, round.height, null()) }
}

fn measure_str(
    font: DrawingFont,
    color: &CGColor,
    pos: Point,
    text: &str,
    bound: Size,
) -> (CFRetained<CTFramesetter>, Rect) {
    let astr = create_attr_str(&font, color, text);
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
    let mut y = pos.y;
    match font.halign {
        HAlign::Center => x -= size.width / 2.0,
        HAlign::Right => x -= size.width,
        _ => {}
    }
    match font.valign {
        VAlign::Center => y -= size.height / 2.0,
        VAlign::Bottom => y -= size.height,
        _ => {}
    }
    (framesetter, Rect::new(Point::new(x, y), size))
}

fn create_attr_str(
    font: &DrawingFont,
    color: &CGColor,
    text: &str,
) -> CFRetained<CFMutableAttributedString> {
    unsafe {
        let mut fontdes = CTFontDescriptor::with_name_and_size(
            NSString::from_str(&font.family).bridge(),
            font.size,
        );

        let mut traits = CTFontSymbolicTraits::empty();
        if font.italic {
            traits |= CTFontSymbolicTraits::TraitItalic;
        }
        if font.bold {
            traits |= CTFontSymbolicTraits::TraitBold;
        }
        if !traits.is_empty() {
            fontdes = fontdes
                .copy_with_symbolic_traits(traits, traits)
                .unwrap_or(fontdes);
        }

        let nfont = CTFont::with_font_descriptor(&fontdes, font.size, null());

        let astr = CFMutableAttributedString::new(kCFAllocatorDefault, 0);
        let text = NSString::from_str(text);
        CFMutableAttributedString::replace_string(
            astr.as_deref(),
            CFRange::new(0, 0),
            Some(text.bridge()),
        );
        CFMutableAttributedString::set_attribute(
            astr.as_deref(),
            CFRange::new(0, text.length() as _),
            Some(kCTFontAttributeName),
            Some(&nfont),
        );
        CFMutableAttributedString::set_attribute(
            astr.as_deref(),
            CFRange::new(0, text.length() as _),
            Some(kCTForegroundColorAttributeName),
            Some(color),
        );
        astr.expect("cannot create CFMutableAttributedString")
    }
}

fn to_cgcolor(c: Color) -> CFRetained<CGColor> {
    unsafe {
        CGColor::new_generic_rgb(
            c.r as f64 / 255.0,
            c.g as f64 / 255.0,
            c.b as f64 / 255.0,
            c.a as f64 / 255.0,
        )
    }
}

fn real_point(p: RelativePoint, rect: NSRect) -> NSPoint {
    let p = NSPoint::new(p.x, 1.0 - p.y);
    NSPoint::new(
        rect.origin.x + rect.size.width * p.x,
        rect.origin.y + rect.size.height * p.y,
    )
}

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction;

    #[doc(hidden)]
    fn text_color(&self) -> CFRetained<CGColor>;

    #[doc(hidden)]
    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> DrawAction;
}

impl<B: Brush> Brush for &'_ B {
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        (**self).create_action(path)
    }

    fn text_color(&self) -> CFRetained<CGColor> {
        (**self).text_color()
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> DrawAction {
        (**self).create_text_action(framesetter, rect)
    }
}

impl Brush for SolidColorBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        DrawAction::Path(path, to_cgcolor(self.color), None)
    }

    fn text_color(&self) -> CFRetained<CGColor> {
        to_cgcolor(self.color)
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> DrawAction {
        DrawAction::Text(framesetter, rect)
    }
}

unsafe fn create_gradient(stops: &[GradientStop]) -> CFRetained<CGGradient> {
    let colors = CFMutableArray::<CGColor>::with_capacity(stops.len());
    let mut locs = Vec::with_capacity(stops.len());
    for stop in stops {
        let cgcolor = to_cgcolor(stop.color);
        colors.append(cgcolor.as_ref());
        locs.push(stop.pos)
    }
    CGGradient::with_colors(None, Some(colors.bridge()), locs.as_ptr())
        .expect("cannot create CGGradient")
}

fn linear_gradient(b: &LinearGradientBrush, rect: NSRect) -> DrawGradientAction {
    let gradient = unsafe { create_gradient(&b.stops) };
    DrawGradientAction::Linear {
        gradient,
        start_point: real_point(b.start, rect),
        end_point: real_point(b.end, rect),
    }
}

impl Brush for LinearGradientBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        let rect = unsafe { CGPath::bounding_box(Some(&path)) };
        DrawAction::GradientPath(path, linear_gradient(self, rect), None)
    }

    fn text_color(&self) -> CFRetained<CGColor> {
        unsafe { CGColor::constant_color(Some(kCGColorWhite)).unwrap() }
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> DrawAction {
        DrawAction::GradientText(framesetter, linear_gradient(self, rect), rect)
    }
}

fn radial_gradient(b: &RadialGradientBrush, rect: NSRect) -> DrawGradientAction {
    let gradient = unsafe { create_gradient(&b.stops) };
    DrawGradientAction::Radial {
        gradient,
        start_center: real_point(b.origin, rect),
        start_radius: 0.0,
        end_center: real_point(b.center, rect),
        end_radius: (b.radius.width * rect.size.width).max(b.radius.height * rect.size.height),
    }
}

impl Brush for RadialGradientBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        let rect = unsafe { CGPath::bounding_box(Some(&path)) };
        DrawAction::GradientPath(path, radial_gradient(self, rect), None)
    }

    fn text_color(&self) -> CFRetained<CGColor> {
        unsafe { CGColor::constant_color(Some(kCGColorWhite)).unwrap() }
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> DrawAction {
        DrawAction::GradientText(framesetter, radial_gradient(self, rect), rect)
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn brush(&self) -> &dyn Brush;
    #[doc(hidden)]
    fn width(&self) -> f64;

    #[doc(hidden)]
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        self.brush().create_action(path).with_width(self.width())
    }
}

impl<P: Pen> Pen for &'_ P {
    fn brush(&self) -> &dyn Brush {
        (**self).brush()
    }

    fn width(&self) -> f64 {
        (**self).width()
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn brush(&self) -> &dyn Brush {
        &self.brush
    }

    fn width(&self) -> f64 {
        self.width
    }
}

#[derive(Debug, Clone)]
pub struct DrawingImage {
    #[allow(unused)]
    buffer: Rc<Vec<u8>>,
    rep: Retained<NSBitmapImageRep>,
}

impl DrawingImage {
    fn new(image: DynamicImage) -> Self {
        let width = image.width();
        let height = image.height();
        let (mut buffer, spp, alpha) = match image {
            DynamicImage::ImageRgb8(_) => (image.into_bytes(), 3, false),
            DynamicImage::ImageRgba8(_) => (image.into_bytes(), 4, true),
            _ => (
                DynamicImage::ImageRgba8(image.into_rgba8()).into_bytes(),
                4,
                true,
            ),
        };
        let mut ptr = buffer.as_mut_ptr();
        let rep = unsafe {
            NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bitmapFormat_bytesPerRow_bitsPerPixel(
                    NSBitmapImageRep::alloc(),
                    &mut ptr,
                    width as _,
                    height as _,
                    8,
                    spp,
                    alpha,
                    false,
                    NSDeviceRGBColorSpace,
                    NSBitmapFormat::empty(),
                    (spp as u32 * width) as _,
                    spp * 8,
                )
                .unwrap()
        };
        Self {
            buffer: Rc::new(buffer),
            rep,
        }
    }

    pub fn size(&self) -> Size {
        from_cgsize(unsafe { self.rep.size() })
    }
}
