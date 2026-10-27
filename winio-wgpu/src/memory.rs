//! Offscreen render target for backends that composite the canvas themselves.
//!
//! The frame is rendered into a canvas-owned texture and then copied back to
//! the CPU, so that the toolkit can draw it with its own painter.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use wgpu::{Buffer, Device, Queue, Texture, TextureFormat, TextureView};

/// Format of the offscreen texture.
pub const FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

/// Format of the view handed to the user, with sRGB encoding.
pub const FORMAT_SRGB: TextureFormat = TextureFormat::Bgra8UnormSrgb;

/// A canvas-owned offscreen texture and the staging buffer used to copy it
/// back to the CPU.
#[derive(Debug, Default)]
pub struct MemoryCanvas {
    frame: Option<Frame>,
}

impl MemoryCanvas {
    /// Create a canvas without a texture.
    pub fn new() -> Self {
        Self::default()
    }

    /// The size of the current frame, if any.
    pub fn size(&self) -> Option<(u32, u32)> {
        self.frame.as_ref().map(|frame| frame.size)
    }

    /// Acquire a frame of `size` physical pixels.
    ///
    /// The texture is recreated when the size changes.
    pub fn acquire(
        &mut self,
        device: &Device,
        size: (u32, u32),
    ) -> (Texture, TextureView, TextureFormat, (u32, u32)) {
        if self.frame.as_ref().map(|frame| frame.size) != Some(size) {
            self.frame = Some(Frame::new(device, size));
        }
        let frame = self.frame.as_ref().expect("just created");
        (
            frame.texture.clone(),
            frame.view.clone(),
            FORMAT_SRGB,
            frame.size,
        )
    }

    /// Copy the last acquired frame back to the CPU.
    ///
    /// The returned pixels are tightly packed `BGRA8`.
    pub fn present(&mut self, queue: &Queue) -> Option<&[u8]> {
        self.frame.as_mut()?.read_back(queue)
    }
}

#[derive(Debug)]
struct Frame {
    device: Device,
    texture: Texture,
    view: TextureView,
    buffer: Buffer,
    size: (u32, u32),
    padded_bytes_per_row: u32,
    pixels: Vec<u8>,
}

impl Frame {
    fn new(device: &Device, (width, height): (u32, u32)) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("winio wgpu canvas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[FORMAT_SRGB],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(FORMAT_SRGB),
            ..Default::default()
        });
        let bytes_per_row = width * 4;
        let aligned = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = bytes_per_row.div_ceil(aligned) * aligned;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("winio wgpu canvas read-back"),
            size: padded_bytes_per_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            device: device.clone(),
            texture,
            view,
            buffer,
            size: (width, height),
            padded_bytes_per_row,
            pixels: Vec::new(),
        }
    }

    fn read_back(&mut self, queue: &Queue) -> Option<&[u8]> {
        let (width, height) = self.size;
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);

        let slice = self.buffer.slice(..);
        let mapped = Arc::new(AtomicBool::new(false));
        let mapped_cb = mapped.clone();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            mapped_cb.store(result.is_ok(), Ordering::Release);
        });
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        if !mapped.load(Ordering::Acquire) {
            return None;
        }

        let view = slice.get_mapped_range().ok()?;
        let bytes_per_row = (width * 4) as usize;
        self.pixels.clear();
        self.pixels.reserve(bytes_per_row * height as usize);
        for row in 0..height as usize {
            let start = row * self.padded_bytes_per_row as usize;
            self.pixels
                .extend_from_slice(&view[start..start + bytes_per_row]);
        }
        drop(view);
        self.buffer.unmap();
        Some(&self.pixels)
    }
}
