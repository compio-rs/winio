use image::DynamicImage;
use winio_primitive::BitmapSize;

use crate::{DrawingContext, DrawingImage, Error, Result, not_impl};

#[derive(Debug, Clone)]
pub struct Image;

impl Image {
    pub fn size(&self) -> Result<BitmapSize> {
        not_impl()
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<DrawingImage> {
        not_impl()
    }
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(_value: DynamicImage) -> Result<Self> {
        not_impl()
    }
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(_value: &DynamicImage) -> Result<Self> {
        not_impl()
    }
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<&DrawingImage> for Image {
    type Error = Error;

    fn try_from(_value: &DrawingImage) -> Result<Self> {
        not_impl()
    }
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<DrawingImage> for Image {
    type Error = Error;

    fn try_from(_value: DrawingImage) -> Result<Self> {
        not_impl()
    }
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<&DrawingImage> for DynamicImage {
    type Error = Error;

    fn try_from(_value: &DrawingImage) -> Result<Self> {
        not_impl()
    }
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<DrawingImage> for DynamicImage {
    type Error = Error;

    fn try_from(_value: DrawingImage) -> Result<Self> {
        not_impl()
    }
}
