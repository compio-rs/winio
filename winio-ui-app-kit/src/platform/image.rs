use std::borrow::Cow;

use image::DynamicImage;
use objc2::{AnyThread, rc::Retained};
use objc2_app_kit::NSImage;
use objc2_foundation::NSSize;
use winio_ui_apple_common::DrawingImage;

use crate::{DrawingContext, Error, Result, catch};

#[derive(Debug, Clone)]
pub struct Image(DrawingImage);

impl Image {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        DrawingImage::new(image).map(Self)
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<DrawingImage> {
        Ok(self.0.clone())
    }

    pub(crate) fn nsimage(&self, height: Option<f64>) -> Result<Retained<NSImage>> {
        let size = self.0.size()?;
        catch(|| {
            let cgimage = self.0.cgimage();
            let size = if let Some(height) = height {
                let scale = height / size.height;
                NSSize::new(size.width * scale, size.height * scale)
            } else {
                NSSize::new(size.width, size.height)
            };
            NSImage::initWithCGImage_size(NSImage::alloc(), cgimage, size)
        })
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: DynamicImage) -> Result<Self> {
        Self::new(Cow::Owned(value))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: &DynamicImage) -> Result<Self> {
        Self::new(Cow::Borrowed(value))
    }
}
