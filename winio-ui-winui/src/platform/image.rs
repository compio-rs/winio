use image::DynamicImage;
use windows::Win32::System::WinRT::IBufferByteAccess;
use windows_core::Interface;
use winui3::Microsoft::UI::Xaml::Media::Imaging::WriteableBitmap;

use crate::Result;

#[derive(Debug)]
pub struct Image(WriteableBitmap);

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        let image = image.into_rgba8();
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

    pub(crate) fn as_ref(&self) -> &WriteableBitmap {
        &self.0
    }
}
