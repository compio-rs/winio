use std::ptr::{null, null_mut};

use compio_log::error;
use image::DynamicImage;
use objc2_core_foundation::{
    CFMutableArray, CFMutableAttributedString, CFRange, CFRetained, CGAffineTransform,
    kCFAllocatorDefault,
};
use objc2_core_graphics::{
    CGAffineTransformIsIdentity, CGBitmapContextCreate, CGBitmapContextCreateImage, CGColor,
    CGColorSpace, CGContext, CGGradient, CGGradientDrawingOptions, CGImage, CGImageAlphaInfo,
    CGPath, kCGColorWhite,
};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontSymbolicTraits, CTFramesetter, kCTFontAttributeName,
    kCTForegroundColorAttributeName,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use winio_primitive::{
    BrushPen, Color, DrawingFont, GradientStop, LinearGradientBrush, Point, RadialGradientBrush,
    Rect, RelativePoint, Size, SolidColorBrush,
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

#[derive(Debug)]
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
    pub fn draw(&self, context: &CGContext) {
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
pub enum DrawAction {
    Path(CFRetained<CGPath>, CFRetained<CGColor>, Option<f64>),
    GradientPath(CFRetained<CGPath>, DrawGradientAction, Option<f64>),
    Text(CFRetained<CTFramesetter>, NSRect),
    GradientText(CFRetained<CTFramesetter>, DrawGradientAction, NSRect),
    Image(DrawingImage, NSRect, Option<NSRect>),
    Transform(CGAffineTransform),
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

    pub fn draw_rect(actions: &[Self], context: &CGContext, factor: f64) {
        let mut current_transform = None;
        for action in actions {
            CGContext::save_g_state(Some(context));
            if let Some(transform) = &current_transform {
                CGContext::concat_ctm(Some(context), *transform);
            }
            match action {
                DrawAction::Path(path, color, width) => {
                    CGContext::add_path(Some(context), Some(path));
                    if let Some(width) = width {
                        CGContext::set_stroke_color_with_color(Some(context), Some(color));
                        CGContext::set_line_width(Some(context), *width);
                        CGContext::stroke_path(Some(context));
                    } else {
                        CGContext::set_fill_color_with_color(Some(context), Some(color));
                        CGContext::fill_path(Some(context));
                    }
                }
                DrawAction::GradientPath(path, gradient, width) => {
                    CGContext::add_path(Some(context), Some(path));
                    if let Some(width) = width {
                        CGContext::set_line_width(Some(context), *width);
                        CGContext::replace_path_with_stroked_path(Some(context));
                        CGContext::clip(Some(context));
                        gradient.draw(context);
                    } else {
                        CGContext::clip(Some(context));
                        gradient.draw(context);
                    }
                }
                DrawAction::Text(framesetter, rect) => unsafe {
                    let text_path = CGPath::with_rect(*rect, null());

                    let frame = framesetter.frame(CFRange::new(0, 0), &text_path, None);

                    frame.draw(context);
                },
                DrawAction::GradientText(framesetter, gradient, rect) => unsafe {
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
                    CGContext::clip_to_mask(Some(context), *rect, mask_image.as_deref());
                    gradient.draw(context);
                },
                DrawAction::Image(image, rect, clip) => {
                    let cg_image = image.cgimage();
                    let clip = if let Some(clip) = clip {
                        CGContext::clip_to_rect(Some(context), *rect);
                        *clip
                    } else {
                        to_cgrect(image.size.into())
                    };
                    let scalex = rect.size.width / clip.size.width;
                    let scaley = rect.size.height / clip.size.height;
                    let real_rect = NSRect::new(
                        NSPoint::new(
                            rect.origin.x - clip.origin.x * scalex,
                            rect.origin.y - clip.origin.y * scaley,
                        ),
                        NSSize::new(image.size.width * scalex, image.size.height * scaley),
                    );
                    CGContext::draw_image(Some(context), real_rect, Some(cg_image));
                }
                DrawAction::Transform(transform) => {
                    if CGAffineTransformIsIdentity(*transform) {
                        current_transform = None;
                    } else {
                        current_transform = Some(*transform);
                    }
                }
            }
            CGContext::restore_g_state(Some(context));
        }
    }
}

pub fn create_attr_str(
    font: &DrawingFont,
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
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<DrawAction>;

    #[doc(hidden)]
    fn text_color(&self) -> Result<CFRetained<CGColor>>;

    #[doc(hidden)]
    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<DrawAction>;
}

impl<B: Brush> Brush for &'_ B {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<DrawAction> {
        (**self).create_action(path)
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        (**self).text_color()
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<DrawAction> {
        (**self).create_text_action(framesetter, rect)
    }
}

impl Brush for SolidColorBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<DrawAction> {
        Ok(DrawAction::Path(path, to_cgcolor(self.color), None))
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        Ok(to_cgcolor(self.color))
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<DrawAction> {
        Ok(DrawAction::Text(framesetter, rect))
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

fn linear_gradient(b: &LinearGradientBrush, rect: NSRect) -> Result<DrawGradientAction> {
    let gradient = create_gradient(&b.stops)?;
    Ok(DrawGradientAction::Linear {
        gradient,
        start_point: real_point(b.start, rect),
        end_point: real_point(b.end, rect),
    })
}

impl Brush for LinearGradientBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<DrawAction> {
        let rect = CGPath::bounding_box(Some(&path));
        Ok(DrawAction::GradientPath(
            path,
            linear_gradient(self, rect)?,
            None,
        ))
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        unsafe { CGColor::constant_color(Some(kCGColorWhite)).ok_or(Error::NullPointer) }
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<DrawAction> {
        Ok(DrawAction::GradientText(
            framesetter,
            linear_gradient(self, rect)?,
            rect,
        ))
    }
}

fn radial_gradient(b: &RadialGradientBrush, rect: NSRect) -> Result<DrawGradientAction> {
    let gradient = create_gradient(&b.stops)?;
    Ok(DrawGradientAction::Radial {
        gradient,
        start_center: real_point(b.origin, rect),
        start_radius: 0.0,
        end_center: real_point(b.center, rect),
        end_radius: (b.radius.width * rect.size.width).max(b.radius.height * rect.size.height),
    })
}

impl Brush for RadialGradientBrush {
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<DrawAction> {
        let rect = CGPath::bounding_box(Some(&path));
        Ok(DrawAction::GradientPath(
            path,
            radial_gradient(self, rect)?,
            None,
        ))
    }

    fn text_color(&self) -> Result<CFRetained<CGColor>> {
        unsafe { CGColor::constant_color(Some(kCGColorWhite)).ok_or(Error::NullPointer) }
    }

    fn create_text_action(
        &self,
        framesetter: CFRetained<CTFramesetter>,
        rect: NSRect,
    ) -> Result<DrawAction> {
        Ok(DrawAction::GradientText(
            framesetter,
            radial_gradient(self, rect)?,
            rect,
        ))
    }
}

/// Drawing pen.
pub trait Pen {
    #[doc(hidden)]
    fn brush(&self) -> &dyn Brush;
    #[doc(hidden)]
    fn width(&self) -> f64;

    #[doc(hidden)]
    fn create_action(&self, path: CFRetained<CGPath>) -> Result<DrawAction> {
        Ok(self.brush().create_action(path)?.with_width(self.width()))
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
    image: CFRetained<CGImage>,
    size: Size,
}

impl DrawingImage {
    pub fn new(image: DynamicImage) -> Result<Self> {
        let width = image.width();
        let height = image.height();
        let size = Size::new(width as f64, height as f64);
        let (mut buffer, spp) = match image {
            DynamicImage::ImageRgba8(_) => (image.into_bytes(), 4),
            _ => (DynamicImage::ImageRgba8(image.into_rgba8()).into_bytes(), 4),
        };
        let ptr = buffer.as_mut_ptr();
        let space = CGColorSpace::new_device_rgb();
        let context = unsafe {
            CGBitmapContextCreate(
                ptr.cast(),
                width as _,
                height as _,
                spp * 2,
                spp * width as usize,
                space.as_deref(),
                CGImageAlphaInfo::PremultipliedLast.0,
            )
        };
        let image = CGBitmapContextCreateImage(context.as_deref()).ok_or(Error::NullPointer)?;
        Ok(Self { image, size })
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.size)
    }

    pub fn cgimage(&self) -> &CGImage {
        &self.image
    }
}
