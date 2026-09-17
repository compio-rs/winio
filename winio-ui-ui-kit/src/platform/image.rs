use image::DynamicImage;
use objc2::rc::Retained;
use objc2_ui_kit::{UIImage, UIImageOrientation, UIImageRenderingMode};
use winio_ui_apple_common::DrawingImage;

use crate::{Result, catch};

#[derive(Debug)]
pub struct Image(DrawingImage);

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        DrawingImage::new(image).map(Self)
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
