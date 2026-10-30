//! WinUI backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(windows)]

pub(crate) use winio_pollable::GlobalRuntime;
pub use winio_ui_windows_common::{Error, Result, Timer};

mod runtime;
pub use runtime::*;

mod widgets;
pub use widgets::*;

mod menu;
pub use menu::*;

mod tool_bar;
pub use tool_bar::*;

mod dialogs;
pub use dialogs::*;

mod platform;
pub use platform::{Clipboard, Data, DataIntoIter, image::Image};

mod hook;

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;
