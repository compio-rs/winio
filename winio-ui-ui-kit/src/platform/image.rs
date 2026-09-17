use std::borrow::Cow;

use image::DynamicImage;
use objc2::rc::Retained;
use objc2_ui_kit::{UIImage, UIImageOrientation, UIImageRenderingMode};
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

    pub(crate) fn uiimage(&self, height: Option<f64>) -> Result<Retained<UIImage>> {
        let size = self.0.size()?;
        catch(|| {
            let cgimage = self.0.cgimage();
            let image = if let Some(height) = height {
                let scale = size.height / height;
                UIImage::imageWithCGImage_scale_orientation(cgimage, scale, UIImageOrientation::Up)
            } else {
                UIImage::imageWithCGImage(cgimage)
            };
            image.imageWithRenderingMode(UIImageRenderingMode::AlwaysOriginal)
        })
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: DynamicImage) -> std::result::Result<Self, Self::Error> {
        Self::new(Cow::Owned(value))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: &DynamicImage) -> std::result::Result<Self, Self::Error> {
        Self::new(Cow::Borrowed(value))
    }
}
