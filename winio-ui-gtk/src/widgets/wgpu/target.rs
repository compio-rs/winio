//! Render targets handed to `wgpu` and composited by GTK.
//!
//! The preferred target is a DMA-BUF backed Vulkan image, which GTK imports
//! with zero copies. When the Vulkan driver cannot export DMA-BUFs (software
//! drivers, for instance), a plain texture plus a read-back is used instead.

use std::{
    fmt,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use ash::vk;
use compio_log::warn;
use gtk4::{Picture, gdk, glib};
use wgpu::hal::{self, vulkan::TextureMemory};
use winio_wgpu::{FrameSink, FrameView, MemoryCanvas};

use crate::{Error, Result};

/// `DRM_FORMAT_ARGB8888`, the DRM name of `VK_FORMAT_B8G8R8A8_UNORM`.
const DRM_FORMAT_ARGB8888: u32 = fourcc(b'A', b'R', b'2', b'4');

/// `DRM_FORMAT_MOD_LINEAR`.
const DRM_FORMAT_MOD_LINEAR: u64 = 0;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

const FORMAT_SRGB: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    a as u32 | (b as u32) << 8 | (c as u32) << 16 | (d as u32) << 24
}

/// A texture owned by the canvas, together with the way to show it.
#[derive(Debug)]
pub(super) enum Target {
    /// The texture memory is a DMA-BUF that GTK imports directly.
    Dmabuf(DmabufTarget),
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
        match DmabufTarget::new(device, size) {
            Ok(target) => Ok(Self::Dmabuf(target)),
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
            Self::Dmabuf(t) => Some(t.size),
            Self::Memory(t) => t.size(),
        }
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        size: (u32, u32),
    ) -> Option<(wgpu::Texture, wgpu::TextureView, wgpu::TextureFormat)> {
        match self {
            Self::Dmabuf(t) => Some((t.texture.clone(), t.view.clone(), FORMAT_SRGB)),
            Self::Memory(t) => t.acquire(device, size),
        }
    }

    pub fn present(&mut self, picture: &Picture, queue: &wgpu::Queue) -> Result<()> {
        match self {
            Self::Dmabuf(t) => t.present(picture, queue),
            Self::Memory(t) => {
                t.present(queue, &mut PictureSink { picture });
                Ok(())
            }
        }
    }
}

/// A `wgpu` texture whose memory is a DMA-BUF that GTK can import.
pub(super) struct DmabufTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: (u32, u32),
    fd: Arc<OwnedFd>,
    stride: u32,
    fourcc: u32,
    modifier: u64,
    device: wgpu::Device,
    // Declared last: the texture owns the image and must be dropped before the
    // device memory backing it.
    _memory: MemoryGuard,
}

impl DmabufTarget {
    fn new(device: &wgpu::Device, size: (u32, u32)) -> std::result::Result<Self, String> {
        let (width, height) = size;

        let hal_device = unsafe { device.as_hal::<hal::api::Vulkan>() }
            .ok_or_else(|| "the wgpu device does not use the Vulkan backend".to_string())?;
        let vk_device = hal_device.raw_device().clone();
        let vk_instance = hal_device.shared_instance().raw_instance();
        let physical = hal_device.raw_physical_device();

        let mut external = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_UNORM)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::LINEAR)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut external);
        let image = unsafe { vk_device.create_image(&image_info, None) }
            .map_err(|e| format!("vkCreateImage failed: {e:?}"))?;

        let requirements = unsafe { vk_device.get_image_memory_requirements(image) };
        let memory_properties =
            unsafe { vk_instance.get_physical_device_memory_properties(physical) };
        let type_index = (0..memory_properties.memory_type_count)
            .find(|i| requirements.memory_type_bits & (1 << i) != 0)
            .ok_or_else(|| "no suitable memory type for the DMA-BUF image".to_string())?;

        let mut export = vk::ExportMemoryAllocateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(type_index)
            .push_next(&mut dedicated)
            .push_next(&mut export);
        let memory = unsafe { vk_device.allocate_memory(&allocate_info, None) }
            .map_err(|e| format!("vkAllocateMemory failed: {e:?}"))?;

        let mut guard = MemoryGuard {
            device: vk_device.clone(),
            image: Some(image),
            memory: Some(memory),
        };

        unsafe { vk_device.bind_image_memory(image, memory, 0) }
            .map_err(|e| format!("vkBindImageMemory failed: {e:?}"))?;

        let raw_fd = {
            let external_memory_fd =
                ash::khr::external_memory_fd::Device::new(vk_instance, &vk_device);
            let info = vk::MemoryGetFdInfoKHR::default()
                .memory(memory)
                .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
            unsafe { external_memory_fd.get_memory_fd(&info) }
                .map_err(|e| format!("vkGetMemoryFdKHR failed: {e:?}"))?
        };
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

        let layout = unsafe {
            vk_device.get_image_subresource_layout(
                image,
                vk::ImageSubresource {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    array_layer: 0,
                },
            )
        };
        let stride = layout.row_pitch as u32;

        let hal_desc = hal::TextureDescriptor {
            label: Some("winio dmabuf canvas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUses::COLOR_TARGET | wgpu::TextureUses::RESOURCE,
            memory_flags: hal::MemoryFlags::empty(),
            view_formats: vec![FORMAT_SRGB],
        };
        let hal_texture =
            unsafe { hal_device.texture_from_raw(image, &hal_desc, None, TextureMemory::External) };
        // The image is owned (and destroyed) by `wgpu` from now on.
        guard.image = None;

        let texture = unsafe {
            device.create_texture_from_hal::<hal::api::Vulkan>(
                hal_texture,
                &texture_descriptor(width, height, wgpu::TextureUsages::RENDER_ATTACHMENT),
                wgpu::TextureUses::UNINITIALIZED,
            )
        };

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(FORMAT_SRGB),
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            size,
            fd: Arc::new(fd),
            stride,
            fourcc: DRM_FORMAT_ARGB8888,
            modifier: DRM_FORMAT_MOD_LINEAR,
            device: device.clone(),
            _memory: guard,
        })
    }

    fn present(&self, picture: &Picture, _queue: &wgpu::Queue) -> Result<()> {
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        // Every frame needs a fresh texture: `GdkDmabufTexture` does not track
        // content changes.
        let display = gdk::Display::default().ok_or(Error::NullPointer)?;
        let fd = self.fd.clone();
        let builder = gdk::DmabufTextureBuilder::new()
            .set_display(&display)
            .set_width(self.size.0)
            .set_height(self.size.1)
            .set_fourcc(self.fourcc)
            .set_modifier(self.modifier)
            .set_n_planes(1)
            .set_offset(0, 0)
            .set_stride(0, self.stride)
            .set_premultiplied(false);
        // SAFETY: `fd` is kept alive until GTK releases the texture.
        let builder = unsafe { builder.set_fd(0, fd.as_raw_fd()) };
        // SAFETY: see `set_fd`.
        let texture = unsafe { builder.build_with_release_func(move || drop(fd)) }?;
        picture.set_paintable(Some(&texture));
        Ok(())
    }
}

impl fmt::Debug for DmabufTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DmabufTarget")
            .field("size", &self.size)
            .field("stride", &self.stride)
            .field("fourcc", &self.fourcc)
            .field("modifier", &self.modifier)
            .finish_non_exhaustive()
    }
}

fn texture_descriptor(
    width: u32,
    height: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some("winio canvas"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage,
        view_formats: &[FORMAT_SRGB],
    }
}

/// Owns the Vulkan image and its memory until they are handed to `wgpu`.
struct MemoryGuard {
    device: ash::Device,
    image: Option<vk::Image>,
    memory: Option<vk::DeviceMemory>,
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(image) = self.image.take() {
                self.device.destroy_image(image, None);
            }
            if let Some(memory) = self.memory.take() {
                self.device.free_memory(memory, None);
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
