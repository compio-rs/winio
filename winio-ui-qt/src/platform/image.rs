use std::{borrow::Cow, fmt, rc::Rc};

use cxx::{ExternType, UniquePtr, type_id};
use image::{DynamicImage, Pixel, Rgb, Rgba};
use winio_primitive::Size;

use crate::{DrawingContext, Error, Result};

#[derive(Clone)]
pub struct Image(Rc<ImageData>);

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image").finish_non_exhaustive()
    }
}

struct ImageData {
    #[allow(dead_code)]
    buffer: Vec<u8>,
    image: UniquePtr<ffi::QImage>,
}

impl Image {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        let width = image.width();
        let height = image.height();
        let (format, count, buffer) = match qimage_format(image.as_ref()) {
            Some((format, count)) => {
                let buffer = match image {
                    Cow::Owned(image) => image.into_bytes(),
                    Cow::Borrowed(image) => image.as_bytes().to_vec(),
                };
                (format, count, buffer)
            }
            None => {
                let image = match image {
                    Cow::Owned(image) => image.into_rgba32f(),
                    Cow::Borrowed(image) => image.to_rgba32f(),
                };
                (
                    QImageFormat::RGBA32FPx4,
                    Rgba::<f32>::CHANNEL_COUNT as usize * 4,
                    DynamicImage::ImageRgba32F(image).into_bytes(),
                )
            }
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
        Ok(Self(Rc::new(ImageData { buffer, image })))
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<Self> {
        Ok(self.clone())
    }

    pub fn size(&self) -> Result<Size> {
        let size = self.0.image.size()?;
        Ok(Size::new(size.width as _, size.height as _))
    }

    pub(crate) fn as_qimage(&self) -> &ffi::QImage {
        &self.0.image
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

/// Get the [`QImage`] format of a [`DynamicImage`], if its memory layout can be
/// used directly.
fn qimage_format(image: &DynamicImage) -> Option<(QImageFormat, usize)> {
    match image {
        DynamicImage::ImageRgb8(_) => {
            Some((QImageFormat::RGB888, Rgb::<u8>::CHANNEL_COUNT as usize))
        }
        DynamicImage::ImageRgba8(_) => {
            Some((QImageFormat::RGBA8888, Rgba::<u8>::CHANNEL_COUNT as usize))
        }
        DynamicImage::ImageRgba16(_) => Some((
            QImageFormat::RGBA64,
            Rgba::<u16>::CHANNEL_COUNT as usize * 2,
        )),
        DynamicImage::ImageRgba32F(_) => Some((
            QImageFormat::RGBA32FPx4,
            Rgba::<f32>::CHANNEL_COUNT as usize * 4,
        )),
        _ => None,
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
