use image::DynamicImage;
use objc2::rc::Retained;
use objc2_ui_kit::UIImage;

use crate::{Result, create_cgimage};

#[derive(Debug)]
pub struct Image(Retained<UIImage>);

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        let cgimage = create_cgimage(image)?;
        Ok(Self(UIImage::imageWithCGImage(&cgimage)))
    }

    pub(crate) fn as_uiimage(&self) -> &UIImage {
        &self.0
    }
}
