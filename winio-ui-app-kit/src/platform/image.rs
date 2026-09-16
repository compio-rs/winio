use image::DynamicImage;
use objc2::{AnyThread, rc::Retained};
use objc2_app_kit::NSImage;
use objc2_foundation::NSSize;

use crate::{Result, create_cgimage};

#[derive(Debug)]
pub struct Image(Retained<NSImage>);

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        let size = NSSize::new(image.width() as _, image.height() as _);
        let cgimage = create_cgimage(image)?;
        let image = NSImage::initWithCGImage_size(NSImage::alloc(), &cgimage, size);
        Ok(Self(image))
    }

    pub(crate) fn as_nsimage(&self) -> &NSImage {
        &self.0
    }
}
