use image::DynamicImage;

use crate::{
    sys::{self, Result},
    ui::{DrawingContext, DrawingImage},
};

/// An image that can be used as an icon of widgets.
#[derive(Debug, Clone)]
pub struct Image(pub(crate) sys::Image);

impl Image {
    /// Create a [`DrawingImage`] from the current one.
    pub fn try_to_drawing(&self, context: &DrawingContext) -> Result<DrawingImage> {
        let image = self.0.try_to_drawing(&context.0)?;
        Ok(DrawingImage(image))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = crate::Error;

    fn try_from(value: &DynamicImage) -> Result<Self> {
        let image = sys::Image::try_from(value)?;
        Ok(Self(image))
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = crate::Error;

    fn try_from(value: DynamicImage) -> Result<Self> {
        let image = sys::Image::try_from(value)?;
        Ok(Self(image))
    }
}
