use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use image::DynamicImage;

use crate::{DrawingImage, Error, Result};

/// An image that can be used as an icon of widgets.
#[derive(Debug, Clone)]
pub struct Image(DrawingImage);

impl Deref for Image {
    type Target = DrawingImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: DynamicImage) -> Result<Self> {
        Ok(Self(DrawingImage::new(Cow::Owned(value))?))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: &DynamicImage) -> Result<Self> {
        Ok(Self(DrawingImage::new(Cow::Borrowed(value))?))
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
