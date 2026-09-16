use std::cell::UnsafeCell;

use image::{DynamicImage, imageops::FilterType};
use windows::{
    Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap},
    Storage::Streams::{IBuffer, IBuffer_Impl},
    Win32::System::WinRT::{IBufferByteAccess, IBufferByteAccess_Impl},
};
use windows_core::{Result as WinResult, implement};
use winio_primitive::Size;
use winui3::Microsoft::UI::Xaml::Media::Imaging::SoftwareBitmapSource;

use crate::Result;

#[implement(IBuffer, IBufferByteAccess)]
struct Buffer {
    data: UnsafeCell<Vec<u8>>,
}

impl Buffer {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    fn data(&self) -> &Vec<u8> {
        unsafe { &*self.data.get() }
    }
}

impl IBuffer_Impl for Buffer_Impl {
    fn Capacity(&self) -> WinResult<u32> {
        Ok(self.data().capacity() as _)
    }

    fn Length(&self) -> WinResult<u32> {
        Ok(self.data().len() as _)
    }

    fn SetLength(&self, value: u32) -> WinResult<()> {
        unsafe { (*self.data.get()).resize(value as _, 0) };
        Ok(())
    }
}

impl IBufferByteAccess_Impl for Buffer_Impl {
    fn Buffer(&self) -> WinResult<*mut u8> {
        Ok(unsafe { (*self.data.get()).as_mut_ptr() })
    }
}

#[derive(Debug)]
pub struct Image(DynamicImage);

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        Ok(Self(image))
    }

    pub fn size(&self) -> Result<Size> {
        Ok(Size::new(self.0.width() as _, self.0.height() as _))
    }

    pub fn set_size(&mut self, size: Size) -> Result<()> {
        let width = size.width.max(1.0).round() as u32;
        let height = size.height.max(1.0).round() as u32;
        self.0 = self.0.resize_exact(width, height, FilterType::Triangle);
        Ok(())
    }

    pub(crate) fn source(&self) -> Result<SoftwareBitmapSource> {
        let image = self.0.to_rgba8();
        let (width, height) = image.dimensions();
        let mut data = Vec::with_capacity(image.len());
        for pixel in image.pixels() {
            let [r, g, b, a] = pixel.0;
            data.extend_from_slice(&[
                ((b as u32 * a as u32 + 127) / 255) as u8,
                ((g as u32 * a as u32 + 127) / 255) as u8,
                ((r as u32 * a as u32 + 127) / 255) as u8,
                a,
            ]);
        }
        let buffer: IBuffer = Buffer::new(data).into();
        let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
            &buffer,
            BitmapPixelFormat::Bgra8,
            width as _,
            height as _,
            BitmapAlphaMode::Premultiplied,
        )?;
        let source = SoftwareBitmapSource::new()?;
        source.SetBitmapAsync(&bitmap)?;
        Ok(source)
    }
}
