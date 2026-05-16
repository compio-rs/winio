//! WinUI backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(windows)]

pub(crate) use winio_pollable::GlobalRuntime;
pub use winio_ui_windows_common::{Error, Result};

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

mod hook;

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;
