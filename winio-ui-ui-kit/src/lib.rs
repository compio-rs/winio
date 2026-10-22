//! UIKit backend for winio.
//!
//! The runtime calls `UIApplicationMain` and runs on the main thread.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "ios")]

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;
pub(crate) use winio_pollable::GlobalRuntime;
pub(crate) use winio_ui_apple_common::*;
pub use winio_ui_apple_common::{
    Brush, DrawingContext, DrawingImage, DrawingPath, DrawingPathBuilder, Error, Pen, Result, Timer,
};

mod runtime;
pub use runtime::*;

mod widgets;
pub use widgets::*;

mod dialogs;
pub use dialogs::*;

mod platform;
pub use platform::*;

mod menu;
pub use menu::*;

mod tool_bar;
pub use tool_bar::*;

#[cfg(feature = "compio-compat")]
mod file;
#[cfg(feature = "compio-compat")]
pub use file::*;
