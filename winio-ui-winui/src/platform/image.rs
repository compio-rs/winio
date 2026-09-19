use std::borrow::Cow;

use image::DynamicImage;
use windows::Win32::System::WinRT::IBufferByteAccess;
use windows_core::Interface;
use winio_primitive::to_premultiplied_bgra8;
use winui3::Microsoft::UI::Xaml::Media::Imaging::WriteableBitmap;

use crate::{DrawingContext, DrawingImage, Error, Result};

#[derive(Debug, Clone)]
pub struct Image(WriteableBitmap);

impl Image {
    pub(crate) fn new(image: Cow<'_, DynamicImage>) -> Result<Self> {
        let image = match image {
            Cow::Owned(image) => image.into_rgba8(),
            Cow::Borrowed(image) => image.to_rgba8(),
        };
        let (width, height) = image.dimensions();
        let mut data = image.into_raw();
        to_premultiplied_bgra8(&mut data);
        let bitmap = WriteableBitmap::CreateInstanceWithDimensions(width as _, height as _)?;
        let buffer = bitmap.PixelBuffer()?;
        let access = buffer.cast::<IBufferByteAccess>()?;
        let dst =
            unsafe { std::slice::from_raw_parts_mut(access.Buffer()?, buffer.Length()? as usize) };
        let len = data.len().min(dst.len());
        dst[..len].copy_from_slice(&data[..len]);
        Ok(Self(bitmap))
    }

    pub fn try_to_drawing(&self, context: &DrawingContext) -> Result<DrawingImage> {
        let width = self.0.PixelWidth()? as u32;
        let height = self.0.PixelHeight()? as u32;
        let buffer = self.0.PixelBuffer()?;
        let access = buffer.cast::<IBufferByteAccess>()?;
        let data =
            unsafe { std::slice::from_raw_parts(access.Buffer()?, buffer.Length()? as usize) };
        context.create_image_from_premultiplied_bgra8(width, height, data)
    }

    pub(crate) fn as_ref(&self) -> &WriteableBitmap {
        &self.0
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
        let size = value.size()?;
        let bitmap =
            WriteableBitmap::CreateInstanceWithDimensions(size.width as _, size.height as _)?;
        let buffer = bitmap.PixelBuffer()?;
        let access = buffer.cast::<IBufferByteAccess>()?;
        let dst =
            unsafe { std::slice::from_raw_parts_mut(access.Buffer()?, buffer.Length()? as usize) };
        value.copy_pixels(dst)?;
        Ok(Self(bitmap))
    }
}

impl TryFrom<DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        Self::try_from(&value)
    }
}
