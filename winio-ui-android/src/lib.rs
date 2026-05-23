//! Android backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg(target_os = "android")]

scoped_tls::scoped_thread_local!(pub(crate) static RUNTIME: Runtime);

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;

mod convert;
pub use convert::*;

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

pub type GlobalRef = jni::objects::Global<jni::objects::JObject<'static>>;
