//! Render targets handed to `wgpu` and composited by GTK.
//!
//! The preferred target is a DMA-BUF backed Vulkan image, which GTK imports
//! with zero copies. When the Vulkan driver cannot export DMA-BUFs (software
//! drivers without `udmabuf`, for instance), a plain texture plus a read-back
//! is used instead.

use std::{
    os::fd::AsRawFd,
    sync::atomic::{AtomicBool, Ordering},
};

use compio_log::warn;
use gtk4::{Picture, gdk, glib};
use winio_wgpu::{DmabufImage, FrameSink, FrameView, MemoryCanvas};

use crate::{Error, Result};

/// A texture owned by the canvas, together with the way to show it.
#[derive(Debug)]
pub(super) enum Target {
    /// The texture memory is a DMA-BUF that GTK imports directly.
    Dmabuf(DmabufImage),
    /// The texture is copied back to the CPU and wrapped in a
    /// [`gdk::MemoryTexture`].
    Memory(MemoryCanvas),
}

/// Set once DMA-BUF export is known to be unsupported, so that resizing the
/// canvas does not retry it for every size.
static DMABUF_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

impl Target {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> std::result::Result<Self, String> {
        if DMABUF_UNAVAILABLE.load(Ordering::Relaxed) {
            return Ok(Self::Memory(MemoryCanvas::new()));
        }
        match DmabufImage::new(device, size) {
            Ok(image) => Ok(Self::Dmabuf(image)),
            Err(e) => {
                warn!(
                    "WGPU canvas: cannot export a DMA-BUF ({e}), falling back to a memory texture"
                );
                DMABUF_UNAVAILABLE.store(true, Ordering::Relaxed);
                Ok(Self::Memory(MemoryCanvas::new()))
            }
        }
    }

    pub fn size(&self) -> Option<(u32, u32)> {
        match self {
            Self::Dmabuf(t) => Some(t.size()),
            Self::Memory(t) => t.size(),
        }
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        size: (u32, u32),
    ) -> Option<(wgpu::Texture, wgpu::TextureView, wgpu::TextureFormat)> {
        match self {
            Self::Dmabuf(t) => Some((t.texture().clone(), t.view().clone(), t.format())),
            Self::Memory(t) => t.acquire(device, size),
        }
    }

    pub fn present(&mut self, picture: &Picture, queue: &wgpu::Queue) -> Result<()> {
        match self {
            Self::Dmabuf(t) => {
                // GTK reads the memory directly, so the rendering must be done.
                t.wait();
                let display = gdk::Display::default().ok_or(Error::NullPointer)?;
                let (width, height) = t.size();
                let fd = t.fd();
                let builder = gdk::DmabufTextureBuilder::new()
                    .set_display(&display)
                    .set_width(width)
                    .set_height(height)
                    .set_fourcc(t.fourcc())
                    .set_modifier(t.modifier())
                    .set_n_planes(1)
                    .set_offset(0, 0)
                    .set_stride(0, t.stride())
                    .set_premultiplied(false);
                // SAFETY: `fd` is kept alive until GTK releases the texture.
                let builder = unsafe { builder.set_fd(0, fd.as_raw_fd()) };
                // SAFETY: see `set_fd`.
                let texture = unsafe { builder.build_with_release_func(move || drop(fd)) }?;
                picture.set_paintable(Some(&texture));
                Ok(())
            }
            Self::Memory(t) => {
                t.present(queue, &mut PictureSink { picture });
                Ok(())
            }
        }
    }
}

/// Uploads the frames of a [`MemoryCanvas`] to a [`Picture`].
struct PictureSink<'a> {
    picture: &'a Picture,
}

impl FrameSink for PictureSink<'_> {
    fn release(&mut self) {}

    fn set(&mut self, view: FrameView<'_>) {
        let bytes = glib::Bytes::from_owned(view.pixels.to_vec());
        let texture = gdk::MemoryTexture::new(
            view.size.0 as i32,
            view.size.1 as i32,
            gdk::MemoryFormat::B8g8r8a8,
            &bytes,
            view.stride as usize,
        );
        self.picture.set_paintable(Some(&texture));
    }
}
