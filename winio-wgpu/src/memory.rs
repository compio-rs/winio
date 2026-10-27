//! Offscreen render target for backends that composite the canvas themselves.
//!
//! The frame is rendered into a canvas-owned texture and then copied back to
//! the CPU, so that the toolkit can draw it with its own painter.
//!
//! The copies are pipelined like a swapchain: a few textures and staging
//! buffers are recycled in a ring, so the CPU does not have to wait for the GPU
//! and the pixels can be handed to the toolkit with their padded stride.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use wgpu::{Buffer, Device, Queue, Texture, TextureFormat, TextureView};

use crate::{FORMAT, FORMAT_SRGB};

/// Number of frames that are rendered and copied back concurrently.
const FRAMES: usize = 3;

/// The pixels of a finished frame, as plain `BGRA8`.
#[derive(Debug, Clone, Copy)]
pub struct FrameView<'a> {
    /// The pixels, `size.1` rows of `stride` bytes.
    pub pixels: &'a [u8],
    /// The distance between two rows, in bytes.
    pub stride: u32,
    /// The size of the frame, in physical pixels.
    pub size: (u32, u32),
}

/// Receives the frames of a [`MemoryCanvas`].
pub trait FrameSink {
    /// The pixels handed out last are about to be unmapped.
    ///
    /// Anything that still points into them must be dropped.
    fn release(&mut self);

    /// The pixels of a finished frame, valid until the next [`Self::release`].
    fn set(&mut self, view: FrameView<'_>);
}

/// A canvas-owned offscreen texture and the staging buffers used to copy it
/// back to the CPU.
///
/// Frames are presented with a latency of a few frames: [`present`] submits the
/// frame being rendered and makes the oldest finished frame available through
/// [`current`].
///
/// [`present`]: Self::present
/// [`current`]: Self::current
#[derive(Debug, Default)]
pub struct MemoryCanvas {
    frames: Vec<Frame>,
    /// The frame the user is rendering into.
    acquired: Option<usize>,
    /// The frame whose pixels are available through [`Self::current`].
    current: Option<usize>,
}

impl MemoryCanvas {
    /// Create a canvas without a texture.
    pub fn new() -> Self {
        Self::default()
    }

    /// The size of the frames, if any.
    pub fn size(&self) -> Option<(u32, u32)> {
        self.frames.first().map(|frame| frame.size)
    }

    /// Acquire a frame of `size` physical pixels.
    ///
    /// The textures are recreated when the size changes.
    pub fn acquire(
        &mut self,
        device: &Device,
        size: (u32, u32),
    ) -> Option<(Texture, TextureView, TextureFormat)> {
        if self.size() != Some(size) {
            self.frames.clear();
            self.acquired = None;
            self.current = None;
            for _ in 0..FRAMES {
                self.frames.push(Frame::new(device, size));
            }
        }
        let device = self.frames[0].device.clone();
        self.poll(&device);

        // A frame that was acquired but never presented is dropped.
        if let Some(index) = self.acquired.take()
            && self.frames[index].state == State::Rendering
        {
            self.frames[index].state = State::Free;
        }

        let index = self
            .frames
            .iter()
            .position(|frame| frame.state == State::Free)?;
        self.acquired = Some(index);
        let frame = &mut self.frames[index];
        frame.state = State::Rendering;
        Some((frame.texture.clone(), frame.view.clone(), FORMAT_SRGB))
    }

    /// Submit the frame acquired by [`Self::acquire`] and hand the oldest
    /// finished frame to `sink`.
    ///
    /// Returns whether a frame was handed out.
    pub fn present(&mut self, queue: &Queue, sink: &mut impl FrameSink) -> bool {
        let Some(index) = self.acquired.take() else {
            return false;
        };
        let device = self.frames[index].device.clone();
        queue.submit([self.frames[index].copy_command()]);
        self.frames[index].map();
        self.frames[index].state = State::Mapping;

        self.poll(&device);
        for frame in &mut self.frames {
            if frame.state == State::Mapping && frame.mapped.load(Ordering::Acquire) {
                frame.state = State::Ready;
            }
        }
        let Some(index) = self
            .frames
            .iter()
            .position(|frame| frame.state == State::Ready)
        else {
            return false;
        };

        let previous = self.current.replace(index);
        self.frames[index].state = State::Current;
        if previous.is_some() {
            sink.release();
        }
        if let Some(previous) = previous {
            self.frames[previous].release();
        }

        let frame = &self.frames[index];
        let size = frame.size;
        let stride = frame.stride;
        let Some(mapped) = frame.buffer.slice(..).get_mapped_range().ok() else {
            return false;
        };
        // SAFETY: the buffer stays mapped until `release` is called on the
        // sink, which happens before the frame is unmapped.
        let pixels = unsafe { std::slice::from_raw_parts(mapped.as_ptr(), mapped.len()) };
        sink.set(FrameView {
            pixels,
            stride,
            size,
        });
        true
    }

    fn poll(&self, device: &Device) {
        let _ = device.poll(wgpu::PollType::Poll);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// The texture can be rendered into.
    Free,
    /// The texture is being rendered into.
    Rendering,
    /// The copy was submitted and the buffer is being mapped.
    Mapping,
    /// The pixels are available.
    Ready,
    /// The pixels have been handed to the sink.
    Current,
}

#[derive(Debug)]
struct Frame {
    device: Device,
    texture: Texture,
    view: TextureView,
    buffer: Buffer,
    size: (u32, u32),
    stride: u32,
    mapped: Arc<AtomicBool>,
    state: State,
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
        let stride = bytes_per_row.div_ceil(aligned) * aligned;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("winio wgpu canvas read-back"),
            size: stride as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            device: device.clone(),
            texture,
            view,
            buffer,
            size: (width, height),
            stride,
            mapped: Arc::new(AtomicBool::new(false)),
            state: State::Free,
        }
    }

    fn copy_command(&self) -> wgpu::CommandBuffer {
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
                    bytes_per_row: Some(self.stride),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        encoder.finish()
    }

    /// Map the staging buffer asynchronously; the flag is set when the copy of
    /// the submitted frame has been read back.
    fn map(&self) {
        let mapped = self.mapped.clone();
        mapped.store(false, Ordering::Release);
        self.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                mapped.store(result.is_ok(), Ordering::Release);
            });
    }

    /// Unmap the buffer so that it can be copied into again.
    fn release(&mut self) {
        self.buffer.unmap();
        self.state = State::Free;
    }
}
