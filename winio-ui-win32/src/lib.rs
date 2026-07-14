//! Win32 backend for winio.

#![allow(unused_features)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg(windows)]

pub use winio_ui_windows_common::{Error, Result};

mod runtime;
pub use runtime::*;

mod widgets;
pub use widgets::*;

mod platform;

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;
