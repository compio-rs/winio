//! Shared `wgpu` helpers for winio backends.
//!
//! Two kinds of canvases are provided, behind their own features:
//!
//! * [`SurfaceCanvas`] ([`surface`](crate#features)): for backends where every
//!   widget owns a native surface, so `wgpu` can present to it directly.
//! * [`MemoryCanvas`] ([`memory`](crate#features)): for backends that composite
//!   the widgets themselves; the frame is copied back to the CPU instead.
//! * [`DmabufImage`] ([`dmabuf`](crate#features)): a texture exported as a
//!   DMA-BUF, for toolkits that can import one without a copy.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

/// Format of the textures the canvases render into.
#[cfg(any(feature = "dmabuf", feature = "memory"))]
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// Format of the views handed to the user, with sRGB encoding.
#[cfg(any(feature = "dmabuf", feature = "memory"))]
pub const FORMAT_SRGB: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use memory::*;

#[cfg(feature = "dmabuf")]
mod dmabuf;
#[cfg(feature = "dmabuf")]
pub use dmabuf::*;

#[cfg(feature = "surface")]
mod surface;
#[cfg(feature = "surface")]
pub use surface::*;
