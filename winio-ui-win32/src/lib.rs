//! Win32 backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg(windows)]

pub use std::io::{Error, Result};

scoped_tls::scoped_thread_local!(pub(crate) static RUNTIME: Runtime);

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;
