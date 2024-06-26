use std::{f64::consts::PI, io, rc::Rc};

use core_graphics::{color_space::CGColorSpace, context::CGContext, geometry};
use foreign_types_shared::ForeignType;
use icrate::{
    objc2::{
        class, declare_class, msg_send, msg_send_id,
        mutability::MainThreadOnly,
        rc::{Allocated, Id},
        ClassType, DeclaredClass, Encode, Encoding,
    },
    AppKit::{
        NSBezierPath, NSColor, NSFont, NSFontAttributeName, NSFontDescriptor,
        NSFontDescriptorSymbolicTraits, NSFontDescriptorTraitBold, NSFontDescriptorTraitItalic,
        NSForegroundColorAttributeName, NSGraphicsContext, NSView,
    },
    Foundation::{
        CGPoint, CGRect, MainThreadMarker, NSAffineTransform, NSAttributedString, NSDictionary,
        NSRect, NSString,
    },
};

use super::{callback::Callback, from_cgsize, to_cgsize};
use crate::{
    AsNSView, BrushPen, Color, DrawingFont, HAlign, Margin, Point, Rect, RectBox, Size,
    SolidColorBrush, VAlign, Widget,
};

#[derive(Debug)]
pub struct Canvas {
    view: Id<CanvasView>,
    handle: Widget,
}

impl Canvas {
    pub fn new(parent: impl AsNSView) -> io::Result<Rc<Self>> {
        let view = CanvasView::new(MainThreadMarker::new().unwrap());
        let handle = Widget::from_nsview(parent.as_nsview(), Id::into_super(view.clone()));
        Ok(Rc::new(Self { view, handle }))
    }

    pub fn loc(&self) -> io::Result<Point> {
        self.handle.loc()
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> io::Result<Size> {
        self.handle.size()
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)
    }

    pub fn redraw(&self) -> io::Result<()> {
        unsafe {
            self.handle.as_nsview().setNeedsDisplay(true);
        }
        Ok(())
    }

    pub async fn wait_redraw(&self) -> io::Result<DrawingContext> {
        self.view.ivars().draw_rect.wait().await;
        Ok(DrawingContext::new(self.size()?))
    }
}

impl AsNSView for Canvas {
    fn as_nsview(&self) -> Id<NSView> {
        self.handle.as_nsview()
    }
}

#[derive(Default, Clone)]
struct CanvasViewIvars {
    draw_rect: Callback,
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

        #[method(drawRect:)]
        unsafe fn drawRect(&self, _dirty_rect: NSRect) {
            self.ivars().draw_rect.signal();
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}

pub struct DrawingContext {
    size: Size,
}

impl DrawingContext {
    pub fn new(size: Size) -> Self {
        Self { size }
    }

    pub fn draw_arc(&self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> io::Result<()> {
        let path = path_arc(self.size, rect, start, end);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_pie(&self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> io::Result<()> {
        let path = path_arc(self.size, rect, start, end);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_ellipse(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        let path = path_ellipse(self.size, rect);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_ellipse(&self, brush: impl Brush, rect: Rect) -> io::Result<()> {
        let path = path_ellipse(self.size, rect);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_line(&self, pen: impl Pen, start: Point, end: Point) -> io::Result<()> {
        let rect = RectBox::new(
            Point::new(start.x.min(end.x), start.y.min(end.y)),
            Point::new(start.x.max(end.x), start.y.max(end.y)),
        )
        .to_rect();
        let path = path_line(self.size, start, end);
        pen.draw(&path, self.size, rect)
    }

    pub fn draw_rect(&self, pen: impl Pen, rect: Rect) -> io::Result<()> {
        let path = path_rect(self.size, rect);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_rect(&self, brush: impl Brush, rect: Rect) -> io::Result<()> {
        let path = path_rect(self.size, rect);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_round_rect(&self, pen: impl Pen, rect: Rect, round: Size) -> io::Result<()> {
        let path = path_round_rect(self.size, rect, round);
        pen.draw(&path, self.size, rect)
    }

    pub fn fill_round_rect(&self, brush: impl Brush, rect: Rect, round: Size) -> io::Result<()> {
        let path = path_round_rect(self.size, rect, round);
        brush.draw(&path, self.size, rect)
    }

    pub fn draw_str(
        &self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: impl AsRef<str>,
    ) -> io::Result<()> {
        let (astr, rect) = measure_str(font, pos, text.as_ref())?;
        let location = CGPoint::new(
            rect.origin.x,
            self.size.height - rect.size.height - rect.origin.y,
        );
        draw_mask(
            self.size,
            || unsafe {
                astr.drawAtPoint(location);
                Ok(())
            },
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

fn measure_str(
    font: DrawingFont,
    pos: Point,
    text: &str,
) -> io::Result<(Id<NSAttributedString>, Rect)> {
    let astr = create_attr_str(&font, text)?;
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
    Ok((astr, Rect::new(Point::new(x, y), size)))
}

fn draw_mask(
    s: Size,
    mask: impl FnOnce() -> io::Result<()>,
    fill: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
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

    mask()?;

    unsafe {
        NSGraphicsContext::restoreGraphicsState_class();
    }

    let alpha_mask = mask_context
        .create_image()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot create image"))?;

    let window_context = unsafe {
        let g_context = NSGraphicsContext::currentContext()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot get current CGContext"))?;
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

    fill()?;

    window_context.restore();

    Ok(())
}

fn create_attr_str(font: &DrawingFont, text: &str) -> io::Result<Id<NSAttributedString>> {
    unsafe {
        let mut fontdes = NSFontDescriptor::fontDescriptorWithName_size(
            &NSString::from_str(&font.family),
            font.size,
        );

        let mut traits: NSFontDescriptorSymbolicTraits = 0;
        if font.italic {
            traits |= NSFontDescriptorTraitItalic;
        }
        if font.bold {
            traits |= NSFontDescriptorTraitBold;
        }
        if traits != 0 {
            fontdes = fontdes.fontDescriptorWithSymbolicTraits(traits);
        }

        let nfont = NSFont::fontWithDescriptor_size(&fontdes, font.size)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot create font"))?;

        let attr_str = NSAttributedString::initWithString_attributes(
            NSAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&NSDictionary::from_id_slice(
                &[NSFontAttributeName, NSForegroundColorAttributeName],
                &[Id::cast(nfont), Id::cast(NSColor::whiteColor())],
            )),
        );
        Ok(attr_str)
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
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) -> io::Result<()>;
}

impl<B: Brush> Brush for &'_ B {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) -> io::Result<()> {
        (**self).draw(path, size, rect)
    }
}

impl Brush for SolidColorBrush {
    fn draw(&self, path: &NSBezierPath, _size: Size, _rect: Rect) -> io::Result<()> {
        unsafe {
            to_nscolor(self.color).set();
            path.fill();
        }
        Ok(())
    }
}

pub trait Pen {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) -> io::Result<()>;
}

impl<P: Pen> Pen for &'_ P {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) -> io::Result<()> {
        (**self).draw(path, size, rect)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn draw(&self, path: &NSBezierPath, size: Size, rect: Rect) -> io::Result<()> {
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
                Ok(())
            },
            || self.brush.draw(&region_path, size, rect),
        )
    }
}
