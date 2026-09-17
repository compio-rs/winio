use cxx::{ExternType, UniquePtr, type_id};
use image::{DynamicImage, Pixel, Rgb, Rgba};

use crate::Result;

/// Create a [`QImage`] from a [`DynamicImage`].
///
/// The returned buffer must be kept alive as long as the image, because the
/// image does not own the data.
pub(crate) fn create_image(image: DynamicImage) -> Result<(Vec<u8>, UniquePtr<ffi::QImage>)> {
    let width = image.width();
    let height = image.height();
    let (format, buffer, count) = match image {
        DynamicImage::ImageRgb8(_) => (
            QImageFormat::RGB888,
            image.into_bytes(),
            Rgb::<u8>::CHANNEL_COUNT,
        ),
        DynamicImage::ImageRgba8(_) => (
            QImageFormat::RGBA8888,
            image.into_bytes(),
            Rgba::<u8>::CHANNEL_COUNT,
        ),
        DynamicImage::ImageRgba16(_) => (
            QImageFormat::RGBA64,
            image.into_bytes(),
            Rgba::<u16>::CHANNEL_COUNT * 2,
        ),
        DynamicImage::ImageRgba32F(_) => (
            QImageFormat::RGBA32FPx4,
            image.into_bytes(),
            Rgba::<f32>::CHANNEL_COUNT * 4,
        ),
        _ => (
            QImageFormat::RGBA32FPx4,
            DynamicImage::ImageRgba32F(image.into_rgba32f()).into_bytes(),
            Rgba::<f32>::CHANNEL_COUNT * 4,
        ),
    };
    let image = unsafe {
        ffi::new_image(
            width as _,
            height as _,
            (width * count as u32) as _,
            buffer.as_ptr(),
            format,
        )?
    };
    Ok((buffer, image))
}

pub struct Image {
    #[allow(dead_code)]
    buffer: Vec<u8>,
    image: UniquePtr<ffi::QImage>,
}

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        let (buffer, image) = create_image(image)?;
        Ok(Self { buffer, image })
    }

    pub(crate) fn as_qimage(&self) -> &ffi::QImage {
        &self.image
    }
}

pub use ffi::QImage;

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
pub(crate) enum QImageFormat {
    RGB888     = 13,
    RGBA8888   = 17,
    RGBA64     = 26,
    RGBA32FPx4 = 34,
}

unsafe impl ExternType for QImageFormat {
    type Id = type_id!("QImageFormat");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/platform/image.hpp");

        type QImage;
        type QImageFormat = super::QImageFormat;
        type QSize = crate::widgets::QSize;

        unsafe fn new_image(
            width: i32,
            height: i32,
            stride: i32,
            bits: *const u8,
            format: QImageFormat,
        ) -> Result<UniquePtr<QImage>>;
        fn size(self: &QImage) -> Result<QSize>;
    }
}
