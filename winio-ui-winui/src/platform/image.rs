use std::borrow::Cow;

use image::{DynamicImage, RgbaImage};
use windows::Win32::System::WinRT::IBufferByteAccess;
use windows_core::Interface;
use winui3::Microsoft::UI::Xaml::Media::Imaging::WriteableBitmap;

use crate::{DrawingContext, DrawingImage, Error, Result};

#[derive(Debug, Clone)]
pub struct Image(WriteableBitmap);

impl Image {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        let image: Cow<'_, RgbaImage> = match image {
            Cow::Owned(DynamicImage::ImageRgba8(image)) => Cow::Owned(image),
            Cow::Borrowed(DynamicImage::ImageRgba8(image)) => Cow::Borrowed(image),
            Cow::Owned(image) => Cow::Owned(image.into_rgba8()),
            Cow::Borrowed(image) => Cow::Owned(image.to_rgba8()),
        };
        let (width, height) = image.dimensions();
        let bitmap = WriteableBitmap::CreateInstanceWithDimensions(width as _, height as _)?;
        let buffer = bitmap.PixelBuffer()?;
        let access = buffer.cast::<IBufferByteAccess>()?;
        let data =
            unsafe { std::slice::from_raw_parts_mut(access.Buffer()?, buffer.Length()? as usize) };
        for (pixel, slice) in image.pixels().zip(data.as_chunks_mut::<4>().0) {
            let [r, g, b, a] = pixel.0;
            let a = a as u32;
            slice[0] = ((b as u32 * a + 127) / 255) as u8;
            slice[1] = ((g as u32 * a + 127) / 255) as u8;
            slice[2] = ((r as u32 * a + 127) / 255) as u8;
            slice[3] = a as u8;
        }
        Ok(Self(bitmap))
    }

    pub fn try_to_drawing(&self, context: &DrawingContext) -> Result<DrawingImage> {
        context.create_image_from_premultiplied(self.premultiplied_rgba()?)
    }

    pub(crate) fn as_ref(&self) -> &WriteableBitmap {
        &self.0
    }

    fn pixel_buffer(&self) -> Result<&[u8]> {
        let buffer = self.0.PixelBuffer()?;
        let access = buffer.cast::<IBufferByteAccess>()?;
        let data =
            unsafe { std::slice::from_raw_parts(access.Buffer()?, buffer.Length()? as usize) };
        Ok(data)
    }

    fn premultiplied_rgba(&self) -> Result<RgbaImage> {
        let width = self.0.PixelWidth()? as u32;
        let height = self.0.PixelHeight()? as u32;
        let data = self.pixel_buffer()?;
        let mut rgba = vec![0; width as usize * height as usize * 4];
        for (dst, src) in rgba
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(data.as_chunks::<4>().0)
        {
            // The bitmap is premultiplied BGRA, the drawing image expects premultiplied
            // RGBA.
            dst.copy_from_slice(&[src[2], src[1], src[0], src[3]]);
        }
        Ok(RgbaImage::from_raw(width, height, rgba).expect("invalid image size"))
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
