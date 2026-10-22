//! Android backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "android")]

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;

mod convert;
pub(crate) use convert::*;

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

mod java;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JNI error.
    #[error("JNI error: {0}")]
    Jni(#[from] jni::errors::Error),
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// No application available.
    #[error("No application available")]
    NoApp,
    /// A menu bar already exists.
    #[error("A menu bar already exists")]
    MenuBarExists,
    /// A tool bar already exists.
    #[error("A tool bar already exists")]
    ToolBarExists,
    /// Feature not supported.
    #[error("Feature not supported")]
    NotSupported,
}

pub type Result<T> = std::result::Result<T, Error>;
