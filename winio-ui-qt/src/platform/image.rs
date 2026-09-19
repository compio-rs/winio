use std::{borrow::Cow, fmt, rc::Rc};

use cxx::{ExternType, UniquePtr, type_id};
use image::{DynamicImage, ImageBuffer, Pixel, Rgb, RgbImage, Rgba, RgbaImage};
use winio_primitive::{Size, packed_rows};

use crate::{DrawingContext, Error, Result};

#[derive(Clone)]
pub struct Image(Rc<ImageData>);

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image").finish_non_exhaustive()
    }
}

pub struct DrawingImage(Rc<ImageData>);

struct ImageData {
    #[allow(dead_code)]
    buffer: Vec<u8>,
    image: UniquePtr<ffi::QImage>,
}

impl ImageData {
    fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
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
        Ok(Self { buffer, image })
    }

    /// Create an image filled with transparent pixels.
    fn new_empty(size: Size) -> Result<Self> {
        let width = size.width.round().max(1.0) as i32;
        let height = size.height.round().max(1.0) as i32;
        Ok(Self {
            buffer: Vec::new(),
            image: ffi::new_image_empty(width, height, QImageFormat::RGBA8888)?,
        })
    }

    /// Deep copy the image data.
    fn duplicate(&self) -> Result<Self> {
        Ok(Self {
            buffer: Vec::new(),
            image: ffi::image_copy(&self.image)?,
        })
    }

    fn to_dynamic_image(&self) -> Result<DynamicImage> {
        let image = self.as_qimage();
        let size = image.size()?;
        let width = size.width.max(0) as u32;
        let height = size.height.max(0) as u32;
        if width == 0 || height == 0 {
            return Ok(DynamicImage::new_rgba8(0, 0));
        }
        let stride = ffi::image_bytes_per_line(image);
        let bytes = ffi::image_bytes(image);
        let (w, h) = (width as usize, height as usize);
        Ok(match image.format() {
            QImageFormat::RGB888 => DynamicImage::ImageRgb8(
                RgbImage::from_raw(width, height, packed_rows(bytes, stride, w * 3, h))
                    .expect("invalid image buffer"),
            ),
            QImageFormat::RGBA8888 => DynamicImage::ImageRgba8(
                RgbaImage::from_raw(width, height, packed_rows(bytes, stride, w * 4, h))
                    .expect("invalid image buffer"),
            ),
            QImageFormat::RGBA64 => DynamicImage::ImageRgba16(
                ImageBuffer::<Rgba<u16>, Vec<u16>>::from_raw(
                    width,
                    height,
                    packed_rows(bytes, stride, w * 8, h),
                )
                .expect("invalid image buffer"),
            ),
            QImageFormat::RGBA32FPx4 => DynamicImage::ImageRgba32F(
                ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(
                    width,
                    height,
                    packed_rows(bytes, stride, w * 16, h),
                )
                .expect("invalid image buffer"),
            ),
            _ => {
                let image = ffi::image_to_rgba8(image)?;
                let size = image.size()?;
                let width = size.width.max(0) as u32;
                let height = size.height.max(0) as u32;
                let stride = ffi::image_bytes_per_line(&image);
                DynamicImage::ImageRgba8(
                    RgbaImage::from_raw(
                        width,
                        height,
                        packed_rows(
                            ffi::image_bytes(&image),
                            stride,
                            width as usize * 4,
                            height as usize,
                        ),
                    )
                    .expect("invalid image buffer"),
                )
            }
        })
    }

    fn size(&self) -> Result<Size> {
        let size = self.image.size()?;
        Ok(Size::new(size.width as _, size.height as _))
    }

    fn as_qimage(&self) -> &ffi::QImage {
        &self.image
    }
}

impl Image {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        Ok(Self(Rc::new(ImageData::new(image)?)))
    }

    pub fn try_to_drawing(&self, _context: &DrawingContext) -> Result<DrawingImage> {
        Ok(DrawingImage(self.0.clone()))
    }

    pub(crate) fn as_qimage(&self) -> &ffi::QImage {
        self.0.as_qimage()
    }
}

impl DrawingImage {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        Ok(Self(Rc::new(ImageData::new(image)?)))
    }

    pub(crate) fn new_empty(size: Size) -> Result<Self> {
        Ok(Self(Rc::new(ImageData::new_empty(size)?)))
    }

    pub(crate) fn to_dynamic_image(&self) -> Result<DynamicImage> {
        self.0.to_dynamic_image()
    }

    pub fn size(&self) -> Result<Size> {
        self.0.size()
    }

    pub(crate) fn as_qimage(&self) -> &ffi::QImage {
        self.0.as_qimage()
    }

    /// Make the image data unique (copy-on-write) and return it mutably.
    pub(crate) fn image_mut(&mut self) -> Result<&mut UniquePtr<ffi::QImage>> {
        if Rc::strong_count(&self.0) > 1 {
            self.0 = Rc::new(self.0.duplicate()?);
        }
        Ok(&mut Rc::get_mut(&mut self.0)
            .expect("image data is not shared")
            .image)
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

impl TryFrom<&DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: &DrawingImage) -> Result<Self> {
        Ok(Self(value.0.clone()))
    }
}

impl TryFrom<DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        Ok(Self(value.0))
    }
}

impl TryFrom<&DrawingImage> for DynamicImage {
    type Error = Error;

    fn try_from(value: &DrawingImage) -> Result<Self> {
        value.to_dynamic_image()
    }
}

impl TryFrom<DrawingImage> for DynamicImage {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        value.to_dynamic_image()
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

pub(crate) use ffi::QImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct QImageFormat(pub i32);

#[allow(non_upper_case_globals)]
impl QImageFormat {
    pub const RGB888: Self = Self(13);
    pub const RGBA32FPx4: Self = Self(34);
    pub const RGBA64: Self = Self(26);
    pub const RGBA8888: Self = Self(17);
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
        fn new_image_empty(
            width: i32,
            height: i32,
            format: QImageFormat,
        ) -> Result<UniquePtr<QImage>>;
        fn size(self: &QImage) -> Result<QSize>;
        fn format(self: &QImage) -> QImageFormat;

        fn image_copy(image: &QImage) -> Result<UniquePtr<QImage>>;
        fn image_to_rgba8(image: &QImage) -> Result<UniquePtr<QImage>>;
        fn image_bytes_per_line(image: &QImage) -> usize;
        fn image_bytes(image: &QImage) -> &[u8];
    }
}
