use std::{cell::RefCell, f64::consts::PI, rc::Rc};

use core_graphics::{color_space::CGColorSpace, context::CGContext, geometry};
use foreign_types_shared::ForeignType;
use objc2::{
    ClassType, DeclaredClass, Encode, Encoding, class, declare_class, msg_send, msg_send_id,
    mutability::MainThreadOnly,
    rc::{Allocated, Id},
};
use objc2_app_kit::{
    NSAttributedStringNSStringDrawing, NSBezierPath, NSColor, NSEvent, NSEventType, NSFont,
    NSFontAttributeName, NSFontDescriptor, NSFontDescriptorSymbolicTraits,
    NSForegroundColorAttributeName, NSGraphicsContext, NSTrackingArea, NSTrackingAreaOptions,
    NSView,
};
use objc2_foundation::{
    CGPoint, CGRect, MainThreadMarker, NSAffineTransform, NSAttributedString, NSDictionary,
    NSString,
};

use crate::{
    AsRawWindow, AsWindow, BrushPen, Color, DrawingFont, HAlign, Margin, MouseButton, Point, Rect,
    RectBox, Size, SolidColorBrush, VAlign,
    ui::{Callback, Widget, from_cgsize, to_cgsize},
};

#[derive(Debug)]
pub struct Canvas {
    view: Id<CanvasView>,
    handle: Widget,
}

impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let view = CanvasView::new(MainThreadMarker::new().unwrap());
        let handle = Widget::from_nsview(
            parent.as_window().as_raw_window(),
            Id::into_super(view.clone()),
        );
        Self { view, handle }
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

    fn redraw(&self) {
        unsafe {
            self.view.setNeedsDisplay(true);
        }
    }

    pub fn context(&mut self) -> DrawingContext<'_> {
        DrawingContext {
            size: self.size(),
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
                Point::new(p.x, p.y)
            })
            .unwrap()
    }
}

#[derive(Default, Clone)]
struct CanvasViewIvars {
    mouse_down: Callback<MouseButton>,
    mouse_up: Callback<MouseButton>,
    mouse_move: Callback,
    area: Rc<RefCell<Option<Id<NSTrackingArea>>>>,
}

declare_class! {
    #[derive(Debug)]
    struct CanvasView;

    unsafe impl ClassType for CanvasView {
        type Super = NSView;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "WinioCanvasView";
    }

    impl DeclaredClass for CanvasView {
        type Ivars = CanvasViewIvars;
    }

    #[allow(non_snake_case)]
    unsafe impl CanvasView {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Id<Self>> {
            let this = this.set_ivars(CanvasViewIvars::default());
            unsafe { msg_send_id![super(this), init] }
        }

        #[method(updateTrackingAreas)]
        unsafe fn updateTrackingAreas(&self) {
            let this = self.ivars();
            {
                let mut area = this.area.borrow_mut();
                if let Some(area) = area.take() {
                    self.removeTrackingArea(&area);
                }
                let new_area = NSTrackingArea::initWithRect_options_owner_userInfo(
                    NSTrackingArea::alloc(),
                    self.bounds(),
                    NSTrackingAreaOptions::NSTrackingMouseMoved | NSTrackingAreaOptions::NSTrackingActiveAlways,
                    Some(self),
                    None
                );
                self.addTrackingArea(&new_area);
                *area = Some(new_area);
            }
            msg_send![super(self), updateTrackingAreas]
        }

        #[method(mouseDown:)]
        unsafe fn mouseDown(&self, event: &NSEvent) {
            self.ivars().mouse_down.signal(mouse_button(event));
        }

        #[method(mouseUp:)]
        unsafe fn mouseUp(&self, event: &NSEvent) {
            self.ivars().mouse_up.signal(mouse_button(event));
        }

        #[method(mouseDragged:)]
        unsafe fn mouseDragged(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal(());
        }

        #[method(mouseMoved:)]
        unsafe fn mouseMoved(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal(());
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
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
    canvas: &'a mut Canvas,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        self.canvas.redraw();
    }
}

impl DrawingContext<'_> {
    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) {
        let path = path_arc(self.size, rect, start, end);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) {
        let path = path_ellipse(self.size, rect);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) {
        let path = path_ellipse(self.size, rect);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        let path = path_line(self.size, start, end);
        pen.draw(&path, self.size, rect)
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) {
        let path = path_rect(self.size, rect);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) {
        let path = path_rect(self.size, rect);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) {
        let path = path_round_rect(self.size, rect, round);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) {
        let path = path_round_rect(self.size, rect, round);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) {
        let (astr, rect) = measure_str(font, pos, text.as_ref());
        let location = CGPoint::new(
            rect.origin.x,
            self.size.height - rect.size.height - rect.origin.y,
        );
        draw_mask(
            self.size,
            || unsafe { astr.drawAtPoint(location) },
            || self.fill_rect(brush, rect),
        )
    }
}

fn path_arc(s: Size, rect: Rect, start: f64, end: f64) -> Id<NSBezierPath> {
    unsafe {
        let radius = rect.size / 2.0;
        let centerp = Point::new(rect.origin.x + radius.width, rect.origin.y + radius.height);
        let startp = Point::new(
            centerp.x + radius.width * start.cos(),
            centerp.y + radius.height * start.sin(),
        );

        let rate = radius.height / radius.width;
        let transform = NSAffineTransform::transform();
        transform.translateXBy_yBy(1.0, rate);

        let path = NSBezierPath::bezierPath();
        path.moveToPoint(CGPoint::new(startp.x, (s.height - startp.y) / rate));
        path.appendBezierPathWithArcWithCenter_radius_startAngle_endAngle_clockwise(
            CGPoint::new(centerp.x, (s.height - centerp.y) / rate),
            radius.width,
            -start / PI * 180.0,
            -end / PI * 180.0,
            true,
        );
        path.transformUsingAffineTransform(&transform);
        path
    }
}

fn path_ellipse(s: Size, rect: Rect) -> Id<NSBezierPath> {
    let rect = CGRect::new(
        CGPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    );
    unsafe { NSBezierPath::bezierPathWithOvalInRect(rect) }
}

fn path_line(s: Size, start: Point, end: Point) -> Id<NSBezierPath> {
    unsafe {
        let path = NSBezierPath::bezierPath();
        path.moveToPoint(CGPoint::new(start.x, s.height - start.y));
        path.lineToPoint(CGPoint::new(end.x, s.height - end.y));
        path
    }
}

fn path_rect(s: Size, rect: Rect) -> Id<NSBezierPath> {
    let rect = CGRect::new(
        CGPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    );
    unsafe { NSBezierPath::bezierPathWithRect(rect) }
}

fn path_round_rect(s: Size, rect: Rect, round: Size) -> Id<NSBezierPath> {
    let rect = CGRect::new(
        CGPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    );
    unsafe {
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, round.width, round.height)
    }
}

fn measure_str(font: DrawingFont, pos: Point, text: &str) -> (Id<NSAttributedString>, Rect) {
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

fn draw_mask(s: Size, mask: impl FnOnce(), fill: impl FnOnce()) {
    let colorspace = CGColorSpace::create_device_gray();
    let mask_context = CGContext::create_bitmap_context(
        None,
        s.width as _,
        s.height as _,
        8,
        s.width as _,
        &colorspace,
        0,
    );

    #[repr(transparent)]
    struct CGContextWrapper(*mut core_graphics::sys::CGContext);

    unsafe impl Encode for CGContextWrapper {
        const ENCODING: Encoding = Encoding::Pointer(&Encoding::Struct("CGContext", &[]));
    }

    unsafe {
        let m_context_ptr = CGContextWrapper(mask_context.as_ptr());
        let g_context: Id<NSGraphicsContext> = msg_send_id![class!(NSGraphicsContext), graphicsContextWithCGContext:m_context_ptr flipped:false];
        NSGraphicsContext::saveGraphicsState_class();
        NSGraphicsContext::setCurrentContext(Some(&g_context));
    }

    mask();

    unsafe {
        NSGraphicsContext::restoreGraphicsState_class();
    }

    let alpha_mask = mask_context.create_image().unwrap();

    let window_context = unsafe {
        let g_context = NSGraphicsContext::currentContext().unwrap();
        let ptr: CGContextWrapper = msg_send![&g_context, CGContext];
        CGContext::from_existing_context_ptr(ptr.0)
    };
    window_context.save();
    window_context.clip_to_mask(
        geometry::CGRect::new(
            &geometry::CGPoint::default(),
            &geometry::CGSize::new(s.width, s.height),
        ),
        &alpha_mask,
    );

    fill();

    window_context.restore();
}

fn create_attr_str(font: &DrawingFont, text: &str) -> Id<NSAttributedString> {
    unsafe {
        let mut fontdes = NSFontDescriptor::fontDescriptorWithName_size(
            &NSString::from_str(&font.family),
            font.size,
        );

        let mut traits = NSFontDescriptorSymbolicTraits::empty();
        if font.italic {
            traits |= NSFontDescriptorSymbolicTraits::NSFontDescriptorTraitItalic;
        }
        if font.bold {
            traits |= NSFontDescriptorSymbolicTraits::NSFontDescriptorTraitBold;
        }
        if !traits.is_empty() {
            fontdes = fontdes.fontDescriptorWithSymbolicTraits(traits);
        }

        let nfont = NSFont::fontWithDescriptor_size(&fontdes, font.size).unwrap();

        NSAttributedString::initWithString_attributes(
            NSAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&NSDictionary::from_id_slice(
                &[NSFontAttributeName, NSForegroundColorAttributeName],
                &[Id::cast(nfont), Id::cast(NSColor::whiteColor())],
            )),
        )
    }
}

fn to_nscolor(c: Color) -> Id<NSColor> {
    unsafe {
        NSColor::colorWithCalibratedRed_green_blue_alpha(
            c.r as f64 / 255.0,
            c.g as f64 / 255.0,
            c.b as f64 / 255.0,
            c.a as f64 / 255.0,
        )
    }
}

pub trait Brush {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect);
}

impl<B: Brush> Brush for &'_ B {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) {
        (**self).draw(path, size, rect)
    }
}

impl Brush for SolidColorBrush {
    fn draw(&self, path: &NSBezierPath, _size: Size, _rect: Rect) {
        unsafe {
            to_nscolor(self.color).set();
            path.fill();
        }
    }
}

pub trait Pen {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect);
}

impl<P: Pen> Pen for &'_ P {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) {
        (**self).draw(path, size, rect)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) {
        let region_path = {
            let rect = rect.outer_rect(Margin::new_all_same(self.width));
            path_rect(size, rect)
        };
        draw_mask(
            size,
            || unsafe {
                path.setLineWidth(self.width);
                NSColor::whiteColor().set();
                path.stroke();
            },
            || self.brush.draw(&region_path, size, rect),
        )
    }
}
