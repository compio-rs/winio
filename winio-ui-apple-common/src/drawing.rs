use std::{
    borrow::Cow,
    fmt::{self, Debug},
    mem,
    ptr::{null, null_mut},
    rc::Rc,
};

use compio_log::error;
use image::{DynamicImage, RgbaImage};
use objc2_core_foundation::{
    CFMutableArray, CFMutableAttributedString, CFRange, CFRetained, CGAffineTransform,
    kCFAllocatorDefault,
};
use objc2_core_graphics::{
    CGAffineTransformIsIdentity, CGAffineTransformMake, CGBitmapContextCreate,
    CGBitmapContextCreateImage, CGColor, CGColorSpace, CGContext, CGGradient,
    CGGradientDrawingOptions, CGImage, CGImageAlphaInfo, CGMutablePath, CGPath, kCGColorWhite,
};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontSymbolicTraits, CTFramesetter, kCTFontAttributeName,
    kCTForegroundColorAttributeName,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use winio_primitive::{
    BitmapRect, BitmapSize, BrushPen, Color, Font, GradientStop, LinearGradientBrush, Point,
    RadialGradientBrush, Rect, RelativePoint, Size, SolidColorBrush, Transform,
};

use crate::{Error, Result, TollFreeBridge};

#[inline]
pub fn from_cgsize(size: NSSize) -> Size {
    Size::new(size.width, size.height)
}

#[inline]
pub fn to_cgsize(size: Size) -> NSSize {
    NSSize::new(size.width, size.height)
}

#[inline]
pub fn to_cgrect(rect: Rect) -> NSRect {
    NSRect::new(to_cgpoint(rect.origin), to_cgsize(rect.size))
}

#[inline]
pub fn from_cgrect(rect: NSRect) -> Rect {
    Rect::new(from_cgpoint(rect.origin), from_cgsize(rect.size))
}

#[inline]
pub fn to_cgpoint(p: Point) -> NSPoint {
    NSPoint::new(p.x, p.y)
}

#[inline]
pub fn from_cgpoint(p: NSPoint) -> Point {
    Point::new(p.x, p.y)
}

#[inline]
pub fn transform_rect(s: Size, rect: Rect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        to_cgsize(rect.size),
    )
}

#[inline]
pub fn transform_cgrect(s: Size, rect: NSRect) -> Rect {
    Rect::new(
        Point::new(rect.origin.x, s.height - rect.size.height - rect.origin.y),
        from_cgsize(rect.size),
    )
}

#[inline]
pub fn transform_point(s: Size, p: Point) -> NSPoint {
    NSPoint::new(p.x, s.height - p.y)
}

#[inline]
pub fn transform_cgpoint(s: Size, p: NSPoint) -> Point {
    Point::new(p.x, s.height - p.y)
}

pub struct DrawActionContext<'a> {
    context: &'a CGContext,
    transform: Option<CGAffineTransform>,
    factor: f64,
}

pub trait DrawAction: Debug {
    fn set_width(&mut self, width: f64) {
        let _ = width;
    }

    fn draw(&self, ctx: &mut DrawActionContext);
}

pub fn draw_rect(actions: &[Box<dyn DrawAction>], context: &CGContext, factor: f64) {
    let mut ctx = DrawActionContext {
        context,
        transform: None,
        factor,
    };
    for action in actions {
        CGContext::save_g_state(Some(ctx.context));
        if let Some(transform) = &ctx.transform {
            CGContext::concat_ctm(Some(ctx.context), *transform);
        }
        action.draw(&mut ctx);
        CGContext::restore_g_state(Some(ctx.context));
    }
}

#[derive(Debug)]
struct DrawActionPath {
    path: CFRetained<CGPath>,
    color: CFRetained<CGColor>,
    width: Option<f64>,
}

impl DrawActionPath {
    fn new(path: CFRetained<CGPath>, color: CFRetained<CGColor>, width: Option<f64>) -> Self {
        Self { path, color, width }
    }
}

impl DrawAction for DrawActionPath {
    fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    fn draw(&self, ctx: &mut DrawActionContext) {
        CGContext::add_path(Some(ctx.context), Some(&self.path));
        if let Some(width) = self.width {
            CGContext::set_stroke_color_with_color(Some(ctx.context), Some(&self.color));
            CGContext::set_line_width(Some(ctx.context), width);
            CGContext::stroke_path(Some(ctx.context));
        } else {
            CGContext::set_fill_color_with_color(Some(ctx.context), Some(&self.color));
            CGContext::fill_path(Some(ctx.context));
        }
    }
}

#[derive(Debug)]
enum DrawActionGradientInner {
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

impl DrawActionGradientInner {
    fn draw(&self, context: &CGContext) {
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
struct DrawActionGradient {
    path: CFRetained<CGPath>,
    inner: DrawActionGradientInner,
    width: Option<f64>,
}

impl DrawActionGradient {
    fn new(path: CFRetained<CGPath>, inner: DrawActionGradientInner, width: Option<f64>) -> Self {
        Self { path, inner, width }
    }
}

impl DrawAction for DrawActionGradient {
    fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    fn draw(&self, ctx: &mut DrawActionContext) {
        CGContext::add_path(Some(ctx.context), Some(&self.path));
        if let Some(width) = self.width {
            CGContext::set_line_width(Some(ctx.context), width);
            CGContext::replace_path_with_stroked_path(Some(ctx.context));
        }
        CGContext::clip(Some(ctx.context));
        self.inner.draw(ctx.context);
    }
}

#[derive(Debug)]
struct DrawActionText {
    framesetter: CFRetained<CTFramesetter>,
    rect: NSRect,
}

impl DrawActionText {
    fn new(framesetter: CFRetained<CTFramesetter>, rect: NSRect) -> Self {
        Self { framesetter, rect }
    }
}

impl DrawAction for DrawActionText {
    fn draw(&self, ctx: &mut DrawActionContext) {
        let text_path = unsafe { CGPath::with_rect(self.rect, null()) };
        let frame = unsafe { self.framesetter.frame(CFRange::new(0, 0), &text_path, None) };
        unsafe { frame.draw(ctx.context) };
    }
}

#[derive(Debug)]
struct DrawActionGradientText {
    framesetter: CFRetained<CTFramesetter>,
    inner: DrawActionGradientInner,
    rect: NSRect,
}

impl DrawActionGradientText {
    fn new(
        framesetter: CFRetained<CTFramesetter>,
        inner: DrawActionGradientInner,
        rect: NSRect,
    ) -> Self {
        Self {
            framesetter,
            inner,
            rect,
        }
    }
}

impl DrawAction for DrawActionGradientText {
    fn draw(&self, ctx: &mut DrawActionContext) {
        let colorspace = CGColorSpace::new_device_gray();
        let mask = match unsafe {
            CGBitmapContextCreate(
                null_mut(),
                (self.rect.size.width * ctx.factor) as _,
                (self.rect.size.height * ctx.factor) as _,
                8,
                (self.rect.size.width * ctx.factor) as _,
                colorspace.as_deref(),
                0,
            )
        } {
            Some(context) => context,
            None => {
                error!("Cannot create CGBitmapContext");
                return;
            }
        };

        let text_path =
            unsafe { CGPath::with_rect(NSRect::new(NSPoint::ZERO, self.rect.size), null()) };
        let frame = unsafe { self.framesetter.frame(CFRange::new(0, 0), &text_path, None) };

        CGContext::scale_ctm(Some(&mask), ctx.factor, ctx.factor);
        unsafe { frame.draw(&mask) };

        let mask_image = CGBitmapContextCreateImage(Some(&mask));
        CGContext::clip_to_mask(Some(ctx.context), self.rect, mask_image.as_deref());
        self.inner.draw(ctx.context);
    }
}

#[derive(Debug)]
struct DrawActionImage {
    image: DrawingImage,
    rect: NSRect,
    clip: Option<NSRect>,
}

impl DrawActionImage {
    fn new(image: DrawingImage, rect: NSRect, clip: Option<NSRect>) -> Self {
        Self { image, rect, clip }
    }
}

impl DrawAction for DrawActionImage {
    fn draw(&self, ctx: &mut DrawActionContext) {
        let cg_image = self.image.cgimage();
        let clip = if let Some(clip) = self.clip {
            CGContext::clip_to_rect(Some(ctx.context), self.rect);
            clip
        } else {
            to_cgrect(Rect::from_size(Size::new(
                self.image.size.width as f64,
                self.image.size.height as f64,
            )))
        };
        let scalex = self.rect.size.width / clip.size.width;
        let scaley = self.rect.size.height / clip.size.height;
        let real_rect = NSRect::new(
            NSPoint::new(
                self.rect.origin.x - clip.origin.x * scalex,
                self.rect.origin.y - clip.origin.y * scaley,
            ),
            NSSize::new(
                self.image.size.width as f64 * scalex,
                self.image.size.height as f64 * scaley,
            ),
        );
        CGContext::draw_image(Some(ctx.context), real_rect, Some(cg_image));
    }
}

#[derive(Debug)]
struct DrawActionTransform {
    transform: CGAffineTransform,
}

impl DrawActionTransform {
    fn new(transform: CGAffineTransform) -> Self {
        Self { transform }
    }
}

impl DrawAction for DrawActionTransform {
    fn draw(&self, ctx: &mut DrawActionContext) {
        if CGAffineTransformIsIdentity(self.transform) {
            ctx.transform = None;
        } else {
            ctx.transform = Some(self.transform);
        }
    }
}

pub fn create_attr_str(
    font: &Font,
    color: &CGColor,
    text: &str,
) -> Result<CFRetained<CFMutableAttributedString>> {
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

        let astr =
            CFMutableAttributedString::new(kCFAllocatorDefault, 0).ok_or(Error::NullPointer)?;
        let text = NSString::from_str(text);
        CFMutableAttributedString::replace_string(
            Some(&astr),
            CFRange::new(0, 0),
            Some(text.bridge()),
        );
        CFMutableAttributedString::set_attribute(
            Some(&astr),
            CFRange::new(0, text.length() as _),
            Some(kCTFontAttributeName),
            Some(&nfont),
        );
        CFMutableAttributedString::set_attribute(
            Some(&astr),
            CFRange::new(0, text.length() as _),
            Some(kCTForegroundColorAttributeName),
            Some(color),
        );
        Ok(astr)
    }
}

fn to_cgcolor(c: Color) -> CFRetained<CGColor> {
    CGColor::new_generic_rgb(
        c.r as f64 / 255.0,
        c.g as f64 / 255.0,
        c.b as f64 / 255.0,
        c.a as f64 / 255.0,
    )
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
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<Box<dyn DrawAction>>;

    #[doc(hidden)]
    fn text_color(&self) -> Result<CFRetained<CGColor>>;

    #[doc(hidden)]
    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<Box<dyn DrawAction>>;
}

impl<B: Brush> Brush for &'_ B {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<Box<dyn DrawAction>> {
        (**self).create_action(path)
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        (**self).text_color()
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<Box<dyn DrawAction>> {
        (**self).create_text_action(framesetter, rect)
    }
}

impl Brush for SolidColorBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<Box<dyn DrawAction>> {
        Ok(Box::new(DrawActionPath::new(
            path,
            to_cgcolor(self.color),
            None,
        )))
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        Ok(to_cgcolor(self.color))
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<Box<dyn DrawAction>> {
        Ok(Box::new(DrawActionText::new(framesetter, rect)))
    }
}

fn create_gradient(stops: &[GradientStop]) -> Result<CFRetained<CGGradient>> {
    let colors = CFMutableArray::<CGColor>::with_capacity(stops.len());
    let mut locs = Vec::with_capacity(stops.len());
    for stop in stops {
        let cgcolor = to_cgcolor(stop.color);
        colors.append(cgcolor.as_ref());
        locs.push(stop.pos)
    }
    unsafe {
        CGGradient::with_colors(None, Some(colors.bridge()), locs.as_ptr())
            .ok_or(Error::NullPointer)
    }
}

fn linear_gradient(b: &LinearGradientBrush, rect: NSRect) -> Result<DrawActionGradientInner> {
    let gradient = create_gradient(&b.stops)?;
    Ok(DrawActionGradientInner::Linear {
        gradient,
        start_point: real_point(b.start, rect),
        end_point: real_point(b.end, rect),
    })
}

impl Brush for LinearGradientBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<Box<dyn DrawAction>> {
        let rect = CGPath::bounding_box(Some(&path));
        Ok(Box::new(DrawActionGradient::new(
            path,
            linear_gradient(self, rect)?,
            None,
        )))
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        unsafe { CGColor::constant_color(Some(kCGColorWhite)).ok_or(Error::NullPointer) }
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<Box<dyn DrawAction>> {
        Ok(Box::new(DrawActionGradientText::new(
            framesetter,
            linear_gradient(self, rect)?,
            rect,
        )))
    }
}

fn radial_gradient(b: &RadialGradientBrush, rect: NSRect) -> Result<DrawActionGradientInner> {
    let gradient = create_gradient(&b.stops)?;
    Ok(DrawActionGradientInner::Radial {
        gradient,
        start_center: real_point(b.origin, rect),
        start_radius: 0.0,
        end_center: real_point(b.center, rect),
        end_radius: (b.radius.width * rect.size.width).max(b.radius.height * rect.size.height),
    })
}

impl Brush for RadialGradientBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<Box<dyn DrawAction>> {
        let rect = CGPath::bounding_box(Some(&path));
        Ok(Box::new(DrawActionGradient::new(
            path,
            radial_gradient(self, rect)?,
            None,
        )))
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        unsafe { CGColor::constant_color(Some(kCGColorWhite)).ok_or(Error::NullPointer) }
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<Box<dyn DrawAction>> {
        Ok(Box::new(DrawActionGradientText::new(
            framesetter,
            radial_gradient(self, rect)?,
            rect,
        )))
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn brush(&self) -> &dyn Brush;
    #[doc(hidden)]
    fn width(&self) -> f64;

    #[doc(hidden)]
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<Box<dyn DrawAction>> {
        let mut action = self.brush().create_action(path)?;
        action.set_width(self.width());
        Ok(action)
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

#[derive(Clone)]
pub struct DrawingImage {
    image: CFRetained<CGImage>,
    size: BitmapSize,
    data: Rc<Vec<u8>>,
}

impl fmt::Debug for DrawingImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DrawingImage")
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

/// Premultiply the alpha channel of RGBA pixels.
fn premultiply_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let a = pixel[3] as u16;
        if a == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        } else if a != 255 {
            pixel[0] = ((pixel[0] as u16 * a + 127) / 255) as u8;
            pixel[1] = ((pixel[1] as u16 * a + 127) / 255) as u8;
            pixel[2] = ((pixel[2] as u16 * a + 127) / 255) as u8;
        }
    }
}

/// Unpremultiply the alpha channel of RGBA pixels.
fn unpremultiply_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let a = pixel[3] as u16;
        if a != 0 && a != 255 {
            pixel[0] = ((pixel[0] as u16 * 255 + a / 2) / a).min(255) as u8;
            pixel[1] = ((pixel[1] as u16 * 255 + a / 2) / a).min(255) as u8;
            pixel[2] = ((pixel[2] as u16 * 255 + a / 2) / a).min(255) as u8;
        }
    }
}

/// Create a premultiplied RGBA bitmap context over `data`.
///
/// `data` must stay alive while the context is used.
fn create_context(data: &mut [u8], width: usize, height: usize) -> Result<CFRetained<CGContext>> {
    let space = CGColorSpace::new_device_rgb();
    let context = unsafe {
        CGBitmapContextCreate(
            data.as_mut_ptr().cast(),
            width,
            height,
            8,
            width * 4,
            space.as_deref(),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    };
    context.ok_or(Error::NullPointer)
}

impl DrawingImage {
    /// Create an empty image filled with transparent pixels.
    pub fn new(size: BitmapSize) -> Result<Self> {
        let mut data = vec![0; size.width * size.height * 4];
        let context = create_context(&mut data, size.width, size.height)?;
        let image = CGBitmapContextCreateImage(Some(&context)).ok_or(Error::NullPointer)?;
        Ok(Self {
            image,
            size,
            data: Rc::new(data),
        })
    }

    pub fn from_image(image: Cow<'_, DynamicImage>) -> Result<Self> {
        let width = image.width();
        let height = image.height();
        let size = BitmapSize::new(width as usize, height as usize);
        let mut data = match image {
            Cow::Owned(DynamicImage::ImageRgba8(image)) => image.into_raw(),
            Cow::Borrowed(DynamicImage::ImageRgba8(image)) => image.as_raw().clone(),
            Cow::Owned(image) => image.into_rgba8().into_raw(),
            Cow::Borrowed(image) => image.to_rgba8().into_raw(),
        };
        premultiply_rgba8(&mut data);
        let context = create_context(&mut data, width as _, height as _)?;
        let image = CGBitmapContextCreateImage(Some(&context)).ok_or(Error::NullPointer)?;
        Ok(Self {
            image,
            size,
            data: Rc::new(data),
        })
    }

    /// Get the drawing context, which draws into a private copy of the image.
    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        let width = self.size.width;
        let height = self.size.height;
        let mut data = self.data.as_ref().clone();
        let context = create_context(&mut data, width, height)?;
        Ok(DrawingContext::new_image(
            Size::new(width as f64, height as f64),
            self,
            data,
            context,
        ))
    }

    pub fn size(&self) -> Result<BitmapSize> {
        Ok(self.size)
    }

    pub fn cgimage(&self) -> &CGImage {
        &self.image
    }

    fn to_dynamic_image(&self) -> Result<DynamicImage> {
        let width = self.size.width;
        let height = self.size.height;
        let mut data = self.data.as_ref().clone();
        unpremultiply_rgba8(&mut data);
        Ok(DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width as u32, height as u32, data).expect("invalid image buffer"),
        ))
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

/// The owner of a [`DrawingContext`], which supplies the geometry of the
/// drawing surface and renders the recorded actions.
///
/// The geometry methods default to the AppKit convention of flipping the
/// logical coordinates into a y-up space. Backends with a different convention
/// override the methods that differ.
pub trait ContextOwner {
    /// Create a path for an arc (or a pie, if `pie` is `true`).
    fn path_arc(
        &self,
        size: Size,
        rect: Rect,
        start: f64,
        end: f64,
        pie: bool,
    ) -> CFRetained<CGMutablePath> {
        common_path_arc(size, rect, start, end, pie)
    }

    /// Create a path for an ellipse.
    fn path_ellipse(&self, size: Size, rect: Rect) -> CFRetained<CGPath> {
        common_path_ellipse(size, rect)
    }

    /// Create a path for a line.
    fn path_line(&self, size: Size, start: Point, end: Point) -> CFRetained<CGMutablePath> {
        common_path_line(size, start, end)
    }

    /// Create a path for a rectangle.
    fn path_rect(&self, size: Size, rect: Rect) -> CFRetained<CGPath> {
        common_path_rect(size, rect)
    }

    /// Create a path for a rounded rectangle.
    fn path_round_rect(&self, size: Size, rect: Rect, round: Size) -> CFRetained<CGPath> {
        common_path_round_rect(size, rect, round)
    }

    /// Lay out `text` and return the framesetter with its rect in the drawing
    /// space.
    fn text_frame(
        &self,
        size: Size,
        font: Font,
        color: &CGColor,
        anchor: RelativePoint,
        pos: Point,
        text: &str,
    ) -> Result<(CFRetained<CTFramesetter>, NSRect)> {
        common_text_frame(size, font, color, anchor, pos, text)
    }

    /// The source rectangle of `clip` in the drawing space.
    fn image_clip(&self, image_size: Size, clip: BitmapRect) -> NSRect {
        common_image_clip(image_size, clip)
    }

    /// Finish the drawing with the recorded `actions`.
    fn end_draw(&mut self, actions: Vec<Box<dyn DrawAction>>) -> Result<()>;
}

enum Target<'a> {
    Canvas(&'a mut dyn ContextOwner),
    Image {
        image: &'a mut DrawingImage,
        data: Vec<u8>,
        context: CFRetained<CGContext>,
    },
}

/// Provides the drawing operations of a [`Canvas`](crate::Canvas).
///
/// All recorded geometry is transformed into a y-up space (the
/// `transform_rect`/`transform_point` helpers flip the logical coordinates).
/// AppKit views are not flipped, so their contexts render the actions
/// directly; UIKit views are flipped by `setTransform(scale(1, -1))` while
/// their contexts are y-down, which renders the same y-up actions upright.
/// Image contexts draw the actions on a raw, y-up bitmap context.
pub struct DrawingContext<'a> {
    size: Size,
    actions: Vec<Box<dyn DrawAction>>,
    transform: Transform,
    ended: bool,
    target: Target<'a>,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.end() {
            error!("Error dropping DrawingContext: {e:?}");
        }
    }
}

impl<'a> DrawingContext<'a> {
    pub fn new(
        size: Size,
        owner: &'a mut dyn ContextOwner,
        actions: Vec<Box<dyn DrawAction>>,
    ) -> Self {
        Self {
            size,
            actions,
            transform: Transform::identity(),
            ended: false,
            target: Target::Canvas(owner),
        }
    }

    fn new_image(
        size: Size,
        image: &'a mut DrawingImage,
        data: Vec<u8>,
        context: CFRetained<CGContext>,
    ) -> Self {
        Self {
            size,
            actions: Vec::new(),
            transform: Transform::identity(),
            ended: false,
            target: Target::Image {
                image,
                data,
                context,
            },
        }
    }

    fn end(&mut self) -> Result<()> {
        if self.ended {
            return Ok(());
        }
        self.ended = true;
        match &mut self.target {
            Target::Canvas(owner) => owner.end_draw(mem::take(&mut self.actions)),
            Target::Image {
                image,
                data,
                context,
            } => {
                draw_rect(&self.actions, context, 1.0);
                image.image =
                    CGBitmapContextCreateImage(Some(&*context)).ok_or(Error::NullPointer)?;
                // The image may share the bitmap data with the context, so keep
                // it alive in the image.
                image.data = Rc::new(mem::take(data));
                Ok(())
            }
        }
    }

    pub fn close(mut self) -> Result<()> {
        self.end()
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<()> {
        self.transform = transform;
        self.actions
            .push(Box::new(DrawActionTransform::new(CGAffineTransform {
                a: transform.m11,
                b: transform.m12,
                c: transform.m21,
                d: transform.m22,
                tx: transform.m31,
                ty: transform.m32,
            })));
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
        let path = self.path_arc(rect, start, end, false);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let path = self.path_arc(rect, start, end, true);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> Result<()> {
        let path = self.path_arc(rect, start, end, true);
        self.fill(brush, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let path = self.path_ellipse(rect);
        self.draw(pen, path)
    }

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let path = self.path_ellipse(rect);
        self.fill(brush, path)
    }

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) -> Result<()> {
        let path = self.path_line(start, end);
        self.draw(pen, unsafe { CFRetained::cast_unchecked(path) })
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let path = self.path_rect(rect);
        self.draw(pen, path)
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let path = self.path_rect(rect);
        self.fill(brush, path)
    }

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) -> Result<()> {
        let path = self.path_round_rect(rect, round);
        self.draw(pen, path)
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) -> Result<()> {
        let path = self.path_round_rect(rect, round);
        self.fill(brush, path)
    }

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: Font,
        anchor: RelativePoint,
        pos: Point,
        text: &str,
    ) -> Result<()> {
        let color = brush.text_color()?;
        let (framesetter, rect) = self.text_frame(font, &color, anchor, pos, text)?;
        self.actions
            .push(brush.create_text_action(framesetter, rect)?);
        Ok(())
    }

    pub fn measure_str(&self, font: Font, text: &str) -> Result<Size> {
        let color =
            unsafe { CGColor::constant_color(Some(kCGColorWhite)).ok_or(Error::NullPointer) }?;
        Ok(from_cgsize(
            self.text_frame(font, &color, RelativePoint::zero(), Point::zero(), text)?
                .1
                .size,
        ))
    }

    pub fn create_image(&self, image: Cow<'_, DynamicImage>) -> Result<DrawingImage> {
        DrawingImage::from_image(image)
    }

    pub fn draw_image(
        &mut self,
        image: &DrawingImage,
        rect: Rect,
        clip: Option<BitmapRect>,
    ) -> Result<()> {
        let rect = transform_rect(self.size, rect);
        let clip = match clip {
            Some(clip) => {
                let image_size = image.size()?;
                let image_size = Size::new(image_size.width as f64, image_size.height as f64);
                Some(self.image_clip(image_size, clip))
            }
            None => None,
        };
        self.actions
            .push(Box::new(DrawActionImage::new(image.clone(), rect, clip)));
        Ok(())
    }

    pub fn create_path_builder(&self, start: Point) -> Result<DrawingPathBuilder> {
        Ok(DrawingPathBuilder::new(self.size, start))
    }

    fn path_arc(&self, rect: Rect, start: f64, end: f64, pie: bool) -> CFRetained<CGMutablePath> {
        match &self.target {
            Target::Canvas(owner) => owner.path_arc(self.size, rect, start, end, pie),
            Target::Image { .. } => common_path_arc(self.size, rect, start, end, pie),
        }
    }

    fn path_ellipse(&self, rect: Rect) -> CFRetained<CGPath> {
        match &self.target {
            Target::Canvas(owner) => owner.path_ellipse(self.size, rect),
            Target::Image { .. } => common_path_ellipse(self.size, rect),
        }
    }

    fn path_line(&self, start: Point, end: Point) -> CFRetained<CGMutablePath> {
        match &self.target {
            Target::Canvas(owner) => owner.path_line(self.size, start, end),
            Target::Image { .. } => common_path_line(self.size, start, end),
        }
    }

    fn path_rect(&self, rect: Rect) -> CFRetained<CGPath> {
        match &self.target {
            Target::Canvas(owner) => owner.path_rect(self.size, rect),
            Target::Image { .. } => common_path_rect(self.size, rect),
        }
    }

    fn path_round_rect(&self, rect: Rect, round: Size) -> CFRetained<CGPath> {
        match &self.target {
            Target::Canvas(owner) => owner.path_round_rect(self.size, rect, round),
            Target::Image { .. } => common_path_round_rect(self.size, rect, round),
        }
    }

    fn text_frame(
        &self,
        font: Font,
        color: &CGColor,
        anchor: RelativePoint,
        pos: Point,
        text: &str,
    ) -> Result<(CFRetained<CTFramesetter>, NSRect)> {
        match &self.target {
            Target::Canvas(owner) => owner.text_frame(self.size, font, color, anchor, pos, text),
            Target::Image { .. } => common_text_frame(self.size, font, color, anchor, pos, text),
        }
    }

    fn image_clip(&self, image_size: Size, clip: BitmapRect) -> NSRect {
        match &self.target {
            Target::Canvas(owner) => owner.image_clip(image_size, clip),
            Target::Image { .. } => common_image_clip(image_size, clip),
        }
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

    pub fn add_line(&mut self, p: Point) -> Result<()> {
        let p = transform_point(self.size, p);
        unsafe {
            CGMutablePath::add_line_to_point(Some(&self.path), null(), p.x, p.y);
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
        Ok(())
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) -> Result<()> {
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

fn common_text_frame(
    size: Size,
    font: Font,
    color: &CGColor,
    anchor: RelativePoint,
    pos: Point,
    text: &str,
) -> Result<(CFRetained<CTFramesetter>, NSRect)> {
    let (framesetter, rect) = common_measure_str(font, color, anchor, pos, text, size)?;
    Ok((framesetter, transform_rect(size, rect)))
}

fn common_image_clip(image_size: Size, clip: BitmapRect) -> NSRect {
    transform_rect(
        image_size,
        Rect::new(
            Point::new(clip.origin.x as f64, clip.origin.y as f64),
            Size::new(clip.size.width as f64, clip.size.height as f64),
        ),
    )
}

fn common_path_arc(
    s: Size,
    rect: Rect,
    start: f64,
    end: f64,
    pie: bool,
) -> CFRetained<CGMutablePath> {
    let radius = rect.size / 2.0;
    let centerp = Point::new(rect.origin.x + radius.width, rect.origin.y + radius.height);
    let startp = Point::new(
        centerp.x + radius.width * start.cos(),
        centerp.y + radius.height * start.sin(),
    );

    let rate = radius.height / radius.width;
    let transform = CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0);

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

fn common_path_ellipse(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_ellipse_in_rect(rect, null()) }
}

fn common_path_line(s: Size, start: Point, end: Point) -> CFRetained<CGMutablePath> {
    unsafe {
        let path = CGMutablePath::new();
        let p = transform_point(s, start);
        CGMutablePath::move_to_point(Some(&path), null(), p.x, p.y);
        let p = transform_point(s, end);
        CGMutablePath::add_line_to_point(Some(&path), null(), p.x, p.y);
        path
    }
}

fn common_path_rect(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_rect(rect, null()) }
}

fn common_path_round_rect(s: Size, rect: Rect, round: Size) -> CFRetained<CGPath> {
    let rect = transform_rect(s, rect);
    unsafe { CGPath::with_rounded_rect(rect, round.width, round.height, null()) }
}

fn common_measure_str(
    font: Font,
    color: &CGColor,
    anchor: RelativePoint,
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
    let x = pos.x - size.width * anchor.x;
    let y = pos.y - size.height * anchor.y;
    Ok((framesetter, Rect::new(Point::new(x, y), size)))
}
