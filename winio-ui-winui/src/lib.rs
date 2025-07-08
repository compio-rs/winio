//! WinUI backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg(windows)]

scoped_tls::scoped_thread_local!(pub(crate) static RUNTIME: Runtime);

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;
