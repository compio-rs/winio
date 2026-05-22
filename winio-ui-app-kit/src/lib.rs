//! AppKit backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "macos")]

pub(crate) use winio_pollable::GlobalRuntime;
pub(crate) use winio_ui_apple_common::*;
pub use winio_ui_apple_common::{Error, Result};

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;
