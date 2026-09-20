use std::borrow::Cow;

use image::DynamicImage;
use objc2::rc::Retained;
use objc2_ui_kit::{UIImage, UIImageOrientation, UIImageRenderingMode};
use winio_primitive::BitmapSize;
use winio_ui_apple_common::DrawingImage;

use crate::{DrawingContext, Error, Result, catch};

#[derive(Debug, Clone)]
pub struct Image(DrawingImage);

impl Image {
    pub fn size(&self) -> Result<BitmapSize> {
        self.0.size()
    }

    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        DrawingImage::from_image(image).map(Self)
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<DrawingImage> {
        Ok(self.0.clone())
    }

    pub(crate) fn uiimage(&self, height: Option<f64>) -> Result<Retained<UIImage>> {
        let image_height = self.0.size()?.height as f64;
        catch(|| {
            let cgimage = self.0.cgimage();
            let image = if let Some(height) = height {
                let scale = image_height / height;
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

impl TryFrom<&DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: &DrawingImage) -> Result<Self> {
        Ok(Self(value.clone()))
    }
}

impl TryFrom<DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        Ok(Self(value))
    }
}
