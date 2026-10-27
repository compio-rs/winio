//! Exports `wgpu` textures as DMA-BUFs, so that a toolkit can import them
//! without a copy.
//!
//! The image is created with `VK_EXT_external_memory_dma_buf` and linear
//! tiling, then wrapped as a regular `wgpu` texture. Drivers that cannot export
//! DMA-BUFs (software renderers without `udmabuf`, for instance) return an
//! error from [`DmabufImage::new`], and the caller is expected to fall back to
//! a read-back.

use std::{
    fmt,
    os::fd::{FromRawFd, OwnedFd},
    sync::Arc,
};

use ash::vk;
use wgpu::hal::{self, vulkan::TextureMemory};

use crate::{FORMAT, FORMAT_SRGB};

/// `DRM_FORMAT_ARGB8888`, the DRM name of `VK_FORMAT_B8G8R8A8_UNORM`.
pub const DRM_FORMAT_ARGB8888: u32 = fourcc(b'A', b'R', b'2', b'4');

/// `DRM_FORMAT_MOD_LINEAR`.
pub const DRM_FORMAT_MOD_LINEAR: u64 = 0;

const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    a as u32 | (b as u32) << 8 | (c as u32) << 16 | (d as u32) << 24
}

/// A `wgpu` texture whose memory is a DMA-BUF that the toolkit can import.
pub struct DmabufImage {
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

impl DmabufImage {
    /// Create a DMA-BUF backed texture of `size` physical pixels.
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Result<Self, String> {
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
                &wgpu::TextureDescriptor {
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
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[FORMAT_SRGB],
                },
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

    /// The texture to render into.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// A view of [`texture`](Self::texture), to be used as the render target.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// The format of [`view`](Self::view).
    pub fn format(&self) -> wgpu::TextureFormat {
        FORMAT_SRGB
    }

    /// The size of the image, in physical pixels.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// The DMA-BUF file descriptor, for the toolkit to import.
    pub fn fd(&self) -> Arc<OwnedFd> {
        self.fd.clone()
    }

    /// The distance between two rows, in bytes.
    pub fn stride(&self) -> u32 {
        self.stride
    }

    /// The DRM fourcc code of the pixels.
    pub fn fourcc(&self) -> u32 {
        self.fourcc
    }

    /// The DRM format modifier of the pixels.
    pub fn modifier(&self) -> u64 {
        self.modifier
    }

    /// Wait until the GPU is done with the image.
    ///
    /// The toolkit reads the memory directly, so the rendering must have
    /// finished before the image is handed over.
    pub fn wait(&self) {
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
}

impl fmt::Debug for DmabufImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DmabufImage")
            .field("size", &self.size)
            .field("stride", &self.stride)
            .field("fourcc", &self.fourcc)
            .field("modifier", &self.modifier)
            .finish_non_exhaustive()
    }
}

/// Frees the image and its memory once the `wgpu` texture is gone.
struct MemoryGuard {
    device: ash::Device,
    image: Option<vk::Image>,
    memory: Option<vk::DeviceMemory>,
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(memory) = self.memory.take() {
                self.device.free_memory(memory, None);
            }
            if let Some(image) = self.image.take() {
                self.device.destroy_image(image, None);
            }
        }
    }
}
