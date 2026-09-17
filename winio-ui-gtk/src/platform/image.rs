use std::borrow::Cow;

use gtk4::{
    gdk,
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
};
use image::DynamicImage;

use crate::{DrawingContext, DrawingImage, Error, Result};

#[derive(Debug)]
pub struct Image {
    texture: gdk::Texture,
}

impl Image {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        let image = match image {
            Cow::Owned(image) => image.into_rgba8(),
            Cow::Borrowed(image) => image.to_rgba8(),
        };
        let (width, height) = image.dimensions();
        let bytes = Bytes::from_owned(image.into_raw());
        let pixbuf = Pixbuf::from_bytes(
            &bytes,
            Colorspace::Rgb,
            true,
            8,
            width as _,
            height as _,
            (width * 4) as _,
        );
        let texture = gdk::Texture::for_pixbuf(&pixbuf);
        Ok(Self { texture })
    }

    pub fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            texture: self.texture.clone(),
        })
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<DrawingImage> {
        DrawingImage::from_texture(&self.texture)
    }

    pub fn try_into_drawing(self, context: &DrawingContext) -> Result<DrawingImage> {
        self.try_to_drawing(context)
    }

    pub(crate) fn texture(&self) -> &gdk::Texture {
        &self.texture
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: DynamicImage) -> Result<Self, Self::Error> {
        Self::new(Cow::Owned(value))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: &DynamicImage) -> Result<Self, Self::Error> {
        Self::new(Cow::Borrowed(value))
    }
}
