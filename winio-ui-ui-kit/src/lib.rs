//! UIKit backend for winio.
//!
//! The runtime calls `UIApplicationMain` and runs on the main thread.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "ios")]

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;
pub(crate) use winio_pollable::GlobalRuntime;
pub(crate) use winio_ui_apple_common::*;
pub use winio_ui_apple_common::{Brush, DrawingImage, Error, Pen, Result};

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;
