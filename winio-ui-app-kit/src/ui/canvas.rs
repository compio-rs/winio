use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    ptr::{null, null_mut},
    rc::Rc,
};

use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use objc2::{
    AllocAnyThread, DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{
    NSAttributedStringNSStringDrawing, NSBitmapFormat, NSBitmapImageRep, NSColor,
    NSDeviceRGBColorSpace, NSEvent, NSEventType, NSFont, NSFontAttributeName, NSFontDescriptor,
    NSFontDescriptorSymbolicTraits, NSForegroundColorAttributeName, NSGraphicsContext, NSImage,
    NSView,
};
use objc2_core_foundation::{CFMutableArray, CFRange, CFRetained, CGAffineTransform};
use objc2_core_graphics::{
    CGAffineTransformMake, CGBitmapContextCreate, CGBitmapContextCreateImage, CGColor,
    CGColorSpace, CGContext, CGGradient, CGGradientDrawingOptions,
    CGMutablePath as CGMutablePathRaw, CGPath,
};
use objc2_core_text::CTFramesetter;
use objc2_foundation::{
    MainThreadMarker, NSDictionary, NSMutableAttributedString, NSPoint, NSRange, NSRect, NSString,
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{
    BrushPen, Color, DrawingFont, GradientStop, HAlign, LinearGradientBrush, MouseButton, Point,
    RadialGradientBrush, Rect, RelativePoint, Size, SolidColorBrush, VAlign,
};

use crate::{
    GlobalRuntime, to_cgsize,
    ui::{Widget, from_cgsize, transform_cgpoint, transform_point, transform_rect},
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
            actions: vec![],
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
                    Some(&gradient),
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
                    Some(&gradient),
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
    Text(Retained<NSMutableAttributedString>, NSRect),
    GradientText(
        Retained<NSMutableAttributedString>,
        DrawGradientAction,
        NSRect,
    ),
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

    unsafe fn draw_rect(actions: &[Self], _rect: NSRect, bound: Size) {
        let Some(ns_context) = NSGraphicsContext::currentContext() else {
            return;
        };
        let context = ns_context.CGContext();
        for action in actions {
            CGContext::save_g_state(Some(&context));
            match action {
                Self::Path(path, color, width) => {
                    CGContext::add_path(Some(&context), Some(path));
                    if let Some(width) = width {
                        CGContext::set_stroke_color_with_color(Some(&context), Some(&color));
                        CGContext::set_line_width(Some(&context), *width);
                        CGContext::stroke_path(Some(&context));
                    } else {
                        CGContext::set_fill_color_with_color(Some(&context), Some(&color));
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
                Self::Text(text, rect) => {
                    let text_path = CGPath::with_rect(*rect, null());

                    let framesetter =
                        CTFramesetter::with_attributed_string(&*std::ptr::addr_of!(**text).cast());
                    let frame = framesetter.frame(CFRange::new(0, 0), &text_path, None);

                    frame.draw(&context);
                }
                Self::GradientText(text, gradient, rect) => {
                    let colorspace = CGColorSpace::new_device_gray();
                    let Some(mask) = CGBitmapContextCreate(
                        null_mut(),
                        bound.width as _,
                        bound.height as _,
                        8,
                        bound.width as _,
                        colorspace.as_deref(),
                        0,
                    ) else {
                        continue;
                    };

                    let text_path = CGPath::with_rect(*rect, null());

                    let framesetter =
                        CTFramesetter::with_attributed_string(&*std::ptr::addr_of!(**text).cast());
                    let frame = framesetter.frame(CFRange::new(0, 0), &text_path, None);

                    CGContext::set_gray_fill_color(Some(&mask), 1.0, 1.0);
                    frame.draw(&mask);

                    let mask_image = CGBitmapContextCreateImage(Some(&mask));
                    CGContext::clip_to_mask(
                        Some(&context),
                        NSRect::new(NSPoint::ZERO, to_cgsize(bound)),
                        mask_image.as_deref(),
                    );
                    gradient.draw(&context);
                }
                Self::Image(image, rect, clip) => {
                    let ns_image = NSImage::initWithSize(NSImage::alloc(), image.rep.size());
                    ns_image.addRepresentation(&image.rep);
                    let cg_image =
                        ns_image.CGImageForProposedRect_context_hints(null_mut(), None, None);
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
    size: Cell<Size>,
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
            DrawAction::draw_rect(&*self.ivars().actions.borrow(), rect, self.ivars().size.get())
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
        std::mem::swap(
            &mut *self.canvas.view.ivars().actions.borrow_mut(),
            &mut self.actions,
        );
        self.canvas.view.ivars().size.set(self.canvas.size());
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
        self.draw(pen, path.to_cgpath())
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) {
        self.fill(brush, path.to_cgpath())
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, false);
        self.draw(pen, path.into_cgpath())
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, true);
        self.draw(pen, path.into_cgpath())
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, true);
        self.fill(brush, path.into_cgpath())
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
        self.draw(pen, path.into_cgpath())
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
        let (astr, rect) = measure_str(font, pos, text);
        let rect = transform_rect(self.size, rect);
        self.actions.push(brush.create_text_action(astr, rect))
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

pub struct DrawingPath(CFRetained<objc2_core_graphics::CGMutablePath>);

impl DrawingPath {
    fn to_cgpath(&self) -> CFRetained<CGPath> {
        unsafe { CFRetained::cast_unchecked(self.0.clone()) }
    }
}

pub struct DrawingPathBuilder {
    size: Size,
    path: CGMutablePath,
}

impl DrawingPathBuilder {
    fn new(size: Size, start: Point) -> Self {
        let mut path = CGMutablePath::new();
        path.move_to_point(None, transform_point(size, start));
        Self { size, path }
    }

    pub fn add_line(&mut self, p: Point) {
        self.path.line_to_point(None, transform_point(self.size, p));
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
        self.path.add_arc(
            Some(&transform),
            NSPoint::new(center.x, center.y / rate),
            radius.width,
            -start,
            -end,
            clockwise,
        );
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) {
        self.path.add_curve(
            None,
            transform_point(self.size, p1),
            transform_point(self.size, p2),
            transform_point(self.size, p3),
        );
    }

    pub fn build(mut self, close: bool) -> DrawingPath {
        if close {
            self.path.close();
        }
        DrawingPath(self.path.0)
    }
}

#[inline]
fn to_ptr<T>(v: Option<&T>) -> *const T {
    match v {
        Some(p) => p,
        None => null(),
    }
}

#[repr(transparent)]
struct CGMutablePath(CFRetained<CGMutablePathRaw>);

impl CGMutablePath {
    pub fn new() -> Self {
        Self(unsafe { CGMutablePathRaw::new() })
    }

    pub fn move_to_point(&mut self, transform: Option<&CGAffineTransform>, p: NSPoint) {
        unsafe {
            CGMutablePathRaw::move_to_point(Some(&self.0), to_ptr(transform), p.x, p.y);
        }
    }

    pub fn line_to_point(&mut self, transform: Option<&CGAffineTransform>, p: NSPoint) {
        unsafe {
            CGMutablePathRaw::add_line_to_point(Some(&self.0), to_ptr(transform), p.x, p.y);
        }
    }

    pub fn add_arc(
        &mut self,
        transform: Option<&CGAffineTransform>,
        center: NSPoint,
        radius: f64,
        start: f64,
        end: f64,
        clockwise: bool,
    ) {
        unsafe {
            CGMutablePathRaw::add_arc(
                Some(&self.0),
                to_ptr(transform),
                center.x,
                center.y,
                radius,
                start,
                end,
                clockwise,
            );
        }
    }

    pub fn add_curve(
        &mut self,
        transform: Option<&CGAffineTransform>,
        p1: NSPoint,
        p2: NSPoint,
        p3: NSPoint,
    ) {
        unsafe {
            CGMutablePathRaw::add_curve_to_point(
                Some(&self.0),
                to_ptr(transform),
                p1.x,
                p1.y,
                p2.x,
                p2.y,
                p3.x,
                p3.y,
            );
        }
    }

    pub fn close(&mut self) {
        unsafe {
            CGMutablePathRaw::close_subpath(Some(&self.0));
        }
    }

    pub fn into_cgpath(self) -> CFRetained<CGPath> {
        unsafe { CFRetained::cast_unchecked(self.0) }
    }
}

impl Deref for CGMutablePath {
    type Target = CGPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn path_arc(s: Size, rect: Rect, start: f64, end: f64, pie: bool) -> CGMutablePath {
    let radius = rect.size / 2.0;
    let centerp = Point::new(rect.origin.x + radius.width, rect.origin.y + radius.height);
    let startp = Point::new(
        centerp.x + radius.width * start.cos(),
        centerp.y + radius.height * start.sin(),
    );

    let rate = radius.height / radius.width;
    let transform = unsafe { CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0) };

    let mut path = CGMutablePath::new();
    if pie {
        path.move_to_point(None, transform_point(s, centerp));
        path.line_to_point(None, transform_point(s, startp));
    } else {
        path.move_to_point(None, transform_point(s, startp));
    }
    path.add_arc(
        Some(&transform),
        transform_point(s, centerp),
        radius.width,
        -start,
        -end,
        true,
    );
    if pie {
        path.close();
    }
    path
}

fn path_ellipse(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_ellipse_in_rect(rect, null()) }
}

fn path_line(s: Size, start: Point, end: Point) -> CGMutablePath {
    let mut path = CGMutablePath::new();
    path.move_to_point(None, transform_point(s, start));
    path.line_to_point(None, transform_point(s, end));
    path
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
    pos: Point,
    text: &str,
) -> (Retained<NSMutableAttributedString>, Rect) {
    let astr = create_attr_str(&font, text);
    let size = from_cgsize(unsafe { astr.size() });
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
    (astr, Rect::new(Point::new(x, y), size))
}

fn create_attr_str(font: &DrawingFont, text: &str) -> Retained<NSMutableAttributedString> {
    unsafe {
        let mut fontdes = NSFontDescriptor::fontDescriptorWithName_size(
            &NSString::from_str(&font.family),
            font.size,
        );

        let mut traits = NSFontDescriptorSymbolicTraits::empty();
        if font.italic {
            traits |= NSFontDescriptorSymbolicTraits::TraitItalic;
        }
        if font.bold {
            traits |= NSFontDescriptorSymbolicTraits::TraitBold;
        }
        if !traits.is_empty() {
            fontdes = fontdes.fontDescriptorWithSymbolicTraits(traits);
        }

        let nfont = NSFont::fontWithDescriptor_size(&fontdes, font.size).unwrap();

        NSMutableAttributedString::initWithString_attributes(
            NSMutableAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&NSDictionary::from_slices(
                &[NSFontAttributeName],
                &[nfont.as_ref()],
            )),
        )
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
    fn create_text_action(
        &self,
        text: Retained<NSMutableAttributedString>,
        rect: NSRect,
    ) -> DrawAction;
}

impl<B: Brush> Brush for &'_ B {
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        (**self).create_action(path)
    }

    fn create_text_action(
        &self,
        text: Retained<NSMutableAttributedString>,
        rect: NSRect,
    ) -> DrawAction {
        (**self).create_text_action(text, rect)
    }
}

impl Brush for SolidColorBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> DrawAction {
        DrawAction::Path(path, to_cgcolor(self.color), None)
    }

    fn create_text_action(
        &self,
        text: Retained<NSMutableAttributedString>,
        rect: NSRect,
    ) -> DrawAction {
        unsafe {
            let color = NSColor::colorWithRed_green_blue_alpha(
                self.color.r as f64 / 255.0,
                self.color.g as f64 / 255.0,
                self.color.b as f64 / 255.0,
                self.color.a as f64 / 255.0,
            );
            text.addAttribute_value_range(
                NSForegroundColorAttributeName,
                color.as_ref(),
                NSRange::new(0, text.length() as _),
            );
            DrawAction::Text(text, rect)
        }
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
    CGGradient::with_colors(
        None,
        Some(&*std::ptr::addr_of!(**colors).cast()),
        locs.as_ptr(),
    )
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

    fn create_text_action(
        &self,
        text: Retained<NSMutableAttributedString>,
        rect: NSRect,
    ) -> DrawAction {
        unsafe {
            let color = NSColor::whiteColor();
            text.addAttribute_value_range(
                NSForegroundColorAttributeName,
                color.as_ref(),
                NSRange::new(0, text.length() as _),
            );
            DrawAction::GradientText(text, linear_gradient(self, rect), rect)
        }
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

    fn create_text_action(
        &self,
        text: Retained<NSMutableAttributedString>,
        rect: NSRect,
    ) -> DrawAction {
        unsafe {
            let color = NSColor::whiteColor();
            text.addAttribute_value_range(
                NSForegroundColorAttributeName,
                color.as_ref(),
                NSRange::new(0, text.length() as _),
            );
            DrawAction::GradientText(text, radial_gradient(self, rect), rect)
        }
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
                    NSBitmapFormat(0),
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
