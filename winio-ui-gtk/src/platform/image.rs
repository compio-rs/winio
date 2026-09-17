use gtk4::{
    gdk,
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
};
use image::DynamicImage;

use crate::Result;

#[derive(Debug)]
pub struct Image {
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
        Ok(Self { texture })
    }

    pub(crate) fn texture(&self) -> &gdk::Texture {
        &self.texture
    }
}
