use gtk4::{
    gdk,
    gdk_pixbuf::{Colorspace, InterpType, Pixbuf},
    glib::Bytes,
};
use image::DynamicImage;
use winio_primitive::Size;

use crate::{Error, Result};

#[derive(Debug)]
pub struct Image {
    pixbuf: Pixbuf,
    texture: gdk::Texture,
}

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        let image = image.into_rgba8();
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
        Ok(Self { pixbuf, texture })
    }

    pub fn size(&self) -> Result<Size> {
        Ok(Size::new(
            self.pixbuf.width() as _,
            self.pixbuf.height() as _,
        ))
    }

    pub fn set_size(&mut self, size: Size) -> Result<()> {
        self.pixbuf = self
            .pixbuf
            .scale_simple(size.width as _, size.height as _, InterpType::Bilinear)
            .ok_or(Error::NullPointer)?;
        self.texture = gdk::Texture::for_pixbuf(&self.pixbuf);
        Ok(())
    }

    pub(crate) fn texture(&self) -> &gdk::Texture {
        &self.texture
    }
}
