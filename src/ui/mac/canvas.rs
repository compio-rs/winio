use std::{marker::PhantomData, ops::Deref, ptr::null};

use image::{DynamicImage, Pixel, Rgb, Rgba};
use objc2::{
    AllocAnyThread, DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
};
use objc2_app_kit::{
    NSAttributedStringNSStringDrawing, NSBitmapFormat, NSBitmapImageRep, NSColor,
    NSDeviceRGBColorSpace, NSEvent, NSEventType, NSFont, NSFontAttributeName, NSFontDescriptor,
    NSFontDescriptorSymbolicTraits, NSForegroundColorAttributeName, NSImage, NSView,
};
use objc2_core_foundation::{CFRetained, CFString, CGAffineTransform};
use objc2_core_graphics::{
    CGAffineTransformMake, CGColor, CGMutablePath as CGMutablePathRaw, CGPath, kCGColorClear,
    kCGColorWhite,
};
use objc2_foundation::{
    MainThreadMarker, NSAttributedString, NSDictionary, NSMutableArray, NSNumber, NSPoint, NSRect,
    NSString,
};
use objc2_quartz_core::{
    CAGradientLayer, CALayer, CAShapeLayer, CATextLayer, kCAGradientLayerRadial,
};

use crate::{
    AsWindow, BrushPen, Color, DrawingFont, GradientStop, HAlign, LinearGradientBrush, MouseButton,
    Point, RadialGradientBrush, Rect, RectBox, RelativePoint, Size, SolidColorBrush, VAlign,
    ui::{
        Callback, Widget, from_cgsize, to_cgsize, transform_cgpoint, transform_cgrect,
        transform_point, transform_rect,
    },
};

#[derive(Debug)]
pub struct Canvas {
    view: Retained<CanvasView>,
    handle: Widget,
}

impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let view = CanvasView::new(MainThreadMarker::new().unwrap());
        view.setWantsLayer(true);
        let handle = Widget::from_nsview(parent, Retained::into_super(view.clone()));
        Self { view, handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn context(&mut self) -> DrawingContext<'_> {
        unsafe fn clear_layer(layer: &CALayer) {
            if let Some(layers) = layer.sublayers() {
                for layer in layers {
                    clear_layer(&layer);
                }
            }
            layer.setSublayers(None);
        }

        let size = self.size();
        unsafe {
            let layer = self
                .view
                .layer()
                .unwrap_or_else(|| self.view.makeBackingLayer());
            clear_layer(&layer);
            DrawingContext {
                size,
                layer,
                _p: PhantomData,
            }
        }
    }

    pub async fn wait_redraw(&self) {
        std::future::pending().await
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

#[derive(Debug, Default, Clone)]
struct CanvasViewIvars {
    mouse_down: Callback<MouseButton>,
    mouse_up: Callback<MouseButton>,
    mouse_move: Callback,
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

        #[unsafe(method(wantsUpdateLayer))]
        unsafe fn wantsUpdateLayer(&self) -> bool {
            true
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouseDown(&self, event: &NSEvent) {
            self.ivars().mouse_down.signal(mouse_button(event));
        }

        #[unsafe(method(mouseUp:))]
        unsafe fn mouseUp(&self, event: &NSEvent) {
            self.ivars().mouse_up.signal(mouse_button(event));
        }

        #[unsafe(method(mouseDragged:))]
        unsafe fn mouseDragged(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal(());
        }

        #[unsafe(method(mouseMoved:))]
        unsafe fn mouseMoved(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal(());
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
    layer: Retained<CALayer>,
    _p: PhantomData<&'a mut Canvas>,
}

impl DrawingContext<'_> {
    fn pen_draw(&self, pen: impl Pen, path: &CGPath, rect: Rect) {
        let layer = pen.draw(path, self.size, rect);
        self.layer.addSublayer(&layer);
    }

    fn brush_draw(&self, brush: impl Brush, path: &CGPath, rect: Rect) {
        let layer = brush.draw(path, self.size, rect);
        self.layer.addSublayer(&layer);
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) {
        let rect = path.bounding();
        let path = &path.0;
        let rect = transform_cgrect(self.size, rect);
        self.pen_draw(pen, path, rect)
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) {
        let rect = path.bounding();
        let path = &path.0;
        let rect = transform_cgrect(self.size, rect);
        self.brush_draw(brush, path, rect)
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, false);
        self.pen_draw(pen, &path, rect)
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, true);
        self.pen_draw(pen, &path, rect)
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end, true);
        self.brush_draw(brush, &path, rect)
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        let path = path_ellipse(self.size, rect);
        self.pen_draw(pen, &path, rect)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        let path = path_ellipse(self.size, rect);
        self.brush_draw(brush, &path, rect)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        let path = path_line(self.size, start, end);
        self.pen_draw(pen, &path, rect)
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        let path = path_rect(self.size, rect);
        self.pen_draw(pen, &path, rect)
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        let path = path_rect(self.size, rect);
        self.brush_draw(brush, &path, rect)
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        let path = path_round_rect(self.size, rect, round);
        self.pen_draw(pen, &path, rect)
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        let path = path_round_rect(self.size, rect, round);
        self.brush_draw(brush, &path, rect)
    }

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str) {
        unsafe {
            let layer = CATextLayer::new();
            let (astr, rect) = measure_str(font, pos, text);
            layer.setFrame(transform_rect(self.size, rect));
            layer.setString(Some(&astr));
            layer.setWrapped(true);
            let brush_layer = brush.create_layer();
            brush_layer.setFrame(self.layer.bounds());
            brush_layer.setMask(Some(&layer));
            self.layer.addSublayer(&brush_layer);
        }
    }

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage {
        DrawingImage::new(image)
    }

    pub fn draw_image(&mut self, image_rep: &DrawingImage, rect: Rect, clip: Option<Rect>) {
        unsafe {
            let image = NSImage::initWithSize(NSImage::alloc(), image_rep.0.size());
            image.addRepresentation(&image_rep.0);
            let source_layer = CALayer::new();
            source_layer.setContents(Some(&image));
            source_layer.setFrame(transform_rect(self.size, rect));
            let target_layer = CALayer::new();
            target_layer.setFrame(self.layer.bounds());
            if let Some(clip) = clip {
                let mask_layer = CALayer::new();
                mask_layer.setFrame(transform_rect(image_rep.size(), clip));
                let white = CGColor::constant_color(Some(kCGColorWhite));
                mask_layer.setBackgroundColor(white.as_deref());
                source_layer.setMask(Some(&mask_layer));
            }
            target_layer.addSublayer(&source_layer);
            self.layer.addSublayer(&target_layer);
        }
    }

    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder {
        DrawingPathBuilder::new(self.size, start)
    }
}

pub struct DrawingPath(CFRetained<objc2_core_graphics::CGMutablePath>);

impl DrawingPath {
    fn bounding(&self) -> NSRect {
        unsafe { CGPath::bounding_box(Some(&self.0)) }
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
}

impl Deref for CGMutablePath {
    type Target = CGPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe fn to_layer(path: &CGPath) -> Retained<CAShapeLayer> {
    let layer = CAShapeLayer::new();
    layer.setPath(Some(path));
    layer
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

fn measure_str(font: DrawingFont, pos: Point, text: &str) -> (Retained<NSAttributedString>, Rect) {
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

fn create_attr_str(font: &DrawingFont, text: &str) -> Retained<NSAttributedString> {
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
        let fore = NSColor::whiteColor();

        NSAttributedString::initWithString_attributes(
            NSAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&NSDictionary::from_slices(
                &[NSFontAttributeName, NSForegroundColorAttributeName],
                &[nfont.as_ref(), fore.as_ref()],
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

unsafe fn make_layer(
    path: &CGPath,
    brush_layer: &CALayer,
    width: f64,
    size: Size,
    rect: Rect,
    fill: &CFString,
    stroke: &CFString,
) -> Retained<CALayer> {
    let mask_layer = to_layer(path);
    let fill = CGColor::constant_color(Some(fill));
    mask_layer.setFillColor(fill.as_deref());
    let stroke = CGColor::constant_color(Some(stroke));
    mask_layer.setStrokeColor(stroke.as_deref());
    mask_layer.setLineWidth(width);
    let mut brush_rect = transform_rect(size, rect);
    brush_rect.origin.x -= width / 2.0;
    brush_rect.origin.y -= width / 2.0;
    brush_rect.size.width += width;
    brush_rect.size.height += width;
    brush_layer.setFrame(brush_rect);
    let content_layer = CALayer::new();
    content_layer.setFrame(NSRect::new(NSPoint::ZERO, to_cgsize(size)));
    content_layer.addSublayer(brush_layer);
    content_layer.setMask(Some(&mask_layer));
    content_layer
}

/// Drawing brush.
pub trait Brush {
    #[doc(hidden)]
    fn create_layer(&self) -> Retained<CALayer>;

    #[doc(hidden)]
    fn draw(&self, path: &CGPath, size: Size, rect: Rect) -> Retained<CALayer> {
        unsafe {
            make_layer(
                path,
                &self.create_layer(),
                0.0,
                size,
                rect,
                kCGColorWhite,
                kCGColorClear,
            )
        }
    }
}

impl<B: Brush> Brush for &'_ B {
    fn create_layer(&self) -> Retained<CALayer> {
        (**self).create_layer()
    }
}

impl Brush for SolidColorBrush {
    fn create_layer(&self) -> Retained<CALayer> {
        unsafe {
            let layer = CALayer::new();
            let color = to_cgcolor(self.color);
            layer.setBackgroundColor(Some(&color));
            layer
        }
    }
}

unsafe fn create_gradient_layer(
    stops: &[GradientStop],
    start: RelativePoint,
    end: RelativePoint,
    ratio: f64,
) -> Retained<CAGradientLayer> {
    let colors = NSMutableArray::<AnyObject>::new();
    let locs = NSMutableArray::<NSNumber>::new();
    for stop in stops {
        let cgcolor = to_cgcolor(stop.color);
        colors.addObject(cgcolor.as_ref());
        locs.addObject(&NSNumber::new_f64(stop.pos * ratio));
    }
    let gradient = CAGradientLayer::new();
    gradient.setColors(Some(&colors));
    gradient.setLocations(Some(&locs));
    gradient.setStartPoint(NSPoint::new(start.x, 1.0 - start.y));
    gradient.setEndPoint(NSPoint::new(end.x, 1.0 - end.y));
    gradient
}

impl Brush for LinearGradientBrush {
    fn create_layer(&self) -> Retained<CALayer> {
        unsafe {
            let gradient = create_gradient_layer(&self.stops, self.start, self.end, 1.0);
            Retained::cast_unchecked(gradient)
        }
    }
}

impl Brush for RadialGradientBrush {
    fn create_layer(&self) -> Retained<CALayer> {
        unsafe {
            let ratio = self.radius.width.min(self.radius.height);
            let gradient = create_gradient_layer(&self.stops, self.origin, self.center, ratio);
            gradient.setType(kCAGradientLayerRadial);
            Retained::cast_unchecked(gradient)
        }
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn create_layer(&self) -> Retained<CALayer>;
    #[doc(hidden)]
    fn width(&self) -> f64;

    #[doc(hidden)]
    fn draw(&self, path: &CGPath, size: Size, rect: Rect) -> Retained<CALayer> {
        unsafe {
            make_layer(
                path,
                &self.create_layer(),
                self.width(),
                size,
                rect,
                kCGColorClear,
                kCGColorWhite,
            )
        }
    }
}

impl<P: Pen> Pen for &'_ P {
    fn create_layer(&self) -> Retained<CALayer> {
        (**self).create_layer()
    }

    fn width(&self) -> f64 {
        (**self).width()
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create_layer(&self) -> Retained<CALayer> {
        self.brush.create_layer()
    }

    fn width(&self) -> f64 {
        self.width
    }
}

pub struct DrawingImage(Retained<NSBitmapImageRep>);

impl DrawingImage {
    fn new(image: DynamicImage) -> Self {
        let width = image.width();
        let height = image.height();
        let (mut buffer, spp, alpha, ccount) = match image {
            DynamicImage::ImageRgb8(_) => (image.into_bytes(), 3, false, Rgb::<u8>::CHANNEL_COUNT),
            DynamicImage::ImageRgba8(_) => (image.into_bytes(), 4, true, Rgba::<u8>::CHANNEL_COUNT),
            _ => (
                DynamicImage::ImageRgba8(image.into_rgba8()).into_bytes(),
                4,
                true,
                Rgba::<u8>::CHANNEL_COUNT,
            ),
        };
        let mut ptr = buffer.as_mut_ptr();
        unsafe {
            Self(NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bitmapFormat_bytesPerRow_bitsPerPixel(
                    NSBitmapImageRep::alloc(),
                    &mut ptr,
                    width as _,
                    height as _,
                    8,
                    spp,
                    alpha,
                    false,
                    NSDeviceRGBColorSpace,
                    NSBitmapFormat::AlphaNonpremultiplied,
                    (ccount as u32 * width) as _,
                    spp * 8,
                )
                .unwrap()
            )
        }
    }

    pub fn size(&self) -> Size {
        from_cgsize(unsafe { self.0.size() })
    }
}
