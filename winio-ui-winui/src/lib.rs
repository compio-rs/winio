//! WinUI backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(windows)]

use winio_callback::Runnable;
pub use winio_ui_windows_common::{Error, Result};

pub(crate) struct GlobalRuntime;

impl Runnable for GlobalRuntime {
    #[inline]
    fn run() {
        winio_pollable::run_current_task();
    }
}

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

mod hook;

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;
