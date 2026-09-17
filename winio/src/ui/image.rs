use image::DynamicImage;

use crate::{
    sys::{self, Result},
    ui::{DrawingContext, DrawingImage},
};

/// An image that can be used as an icon of widgets.
///
/// The supported image formats are determined by the backend and are not
/// limited to ICO files.
pub struct Image(pub(crate) sys::Image);

impl Image {
    /// Create a new [`Image`] from the current one.
    pub fn try_clone(&self) -> Result<Self> {
        let image = self.0.try_clone()?;
        Ok(Self(image))
    }

    /// Create a [`DrawingImage`] from the current one.
    pub fn try_to_drawing(&self, context: &DrawingContext) -> Result<DrawingImage> {
        let image = self.0.try_to_drawing(&context.0)?;
        Ok(DrawingImage(image))
    }

    /// Convert the current [`Image`] into a [`DrawingImage`].
    pub fn try_into_drawing(self, context: &DrawingContext) -> Result<DrawingImage> {
        let image = self.0.try_into_drawing(&context.0)?;
        Ok(DrawingImage(image))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = crate::Error;

    fn try_from(value: &DynamicImage) -> Result<Self, Self::Error> {
        let image = sys::Image::try_from(value)?;
        Ok(Self(image))
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = crate::Error;

    fn try_from(value: DynamicImage) -> Result<Self, Self::Error> {
        let image = sys::Image::try_from(value)?;
        Ok(Self(image))
    }
}
