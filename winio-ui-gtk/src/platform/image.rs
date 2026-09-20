use std::borrow::Cow;

use gtk4::{gdk, glib::Bytes, prelude::TextureExt};
use image::{DynamicImage, GenericImageView};
use winio_primitive::BitmapSize;

use crate::{DrawingContext, DrawingImage, Error, Result};

fn memory_format(image: &DynamicImage) -> Option<(gdk::MemoryFormat, usize)> {
    match image {
        DynamicImage::ImageRgb8(_) => Some((gdk::MemoryFormat::R8g8b8, 3)),
        DynamicImage::ImageRgba8(_) => Some((gdk::MemoryFormat::R8g8b8a8, 4)),
        DynamicImage::ImageRgb16(_) => Some((gdk::MemoryFormat::R16g16b16, 6)),
        DynamicImage::ImageRgba16(_) => Some((gdk::MemoryFormat::R16g16b16a16, 8)),
        DynamicImage::ImageRgb32F(_) => Some((gdk::MemoryFormat::R32g32b32Float, 12)),
        DynamicImage::ImageRgba32F(_) => Some((gdk::MemoryFormat::R32g32b32a32Float, 16)),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    texture: gdk::Texture,
}

impl Image {
    pub fn size(&self) -> Result<BitmapSize> {
        Ok(BitmapSize::new(
            self.texture.width() as usize,
            self.texture.height() as usize,
        ))
    }

    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        let (width, height) = image.dimensions();
        let (data, format, spp) = match memory_format(&image) {
            Some((format, spp)) => (
                match image {
                    Cow::Owned(image) => image.into_bytes(),
                    Cow::Borrowed(image) => image.as_bytes().to_vec(),
                },
                format,
                spp,
            ),
            None => {
                let image = match image {
                    Cow::Owned(image) => image.into_rgba8(),
                    Cow::Borrowed(image) => image.to_rgba8(),
                };
                (
                    DynamicImage::ImageRgba8(image).into_bytes(),
                    gdk::MemoryFormat::R8g8b8a8,
                    4,
                )
            }
        };
        let bytes = Bytes::from_owned(data);
        let texture = gdk::MemoryTextureBuilder::new()
            .set_bytes(Some(&bytes))
            .set_format(format)
            .set_width(width as i32)
            .set_height(height as i32)
            .set_stride(width as usize * spp)
            .build();
        Ok(Self { texture })
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<DrawingImage> {
        DrawingImage::from_texture(&self.texture)
    }

    pub(crate) fn texture(&self) -> &gdk::Texture {
        &self.texture
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
