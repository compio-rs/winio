//! WinUI backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(windows)]

use winio_callback::Runnable;

pub(crate) struct GlobalRuntime;

impl Runnable for GlobalRuntime {
    #[inline]
    fn run() {
        RUNTIME.with(|runtime| runtime.run());
    }
}

scoped_tls::scoped_thread_local!(pub(crate) static RUNTIME: Runtime);

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;
