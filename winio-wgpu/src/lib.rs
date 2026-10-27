//! Shared `wgpu` helpers for winio backends.
//!
//! Two kinds of canvases are provided, behind their own features:
//!
//! * [`SurfaceCanvas`] ([`surface`](crate#features)): for backends where every
//!   widget owns a native surface, so `wgpu` can present to it directly.
//! * [`MemoryCanvas`] ([`memory`](crate#features)): for backends that composite
//!   the widgets themselves; the frame is copied back to the CPU instead.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use memory::*;

#[cfg(feature = "surface")]
mod surface;
#[cfg(feature = "surface")]
pub use surface::*;
