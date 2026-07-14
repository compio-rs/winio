#[cfg(feature = "content-dialog")]
mod msgbox;
#[cfg(feature = "content-dialog")]
pub use msgbox::*;
#[cfg(not(feature = "content-dialog"))]
pub use winio_ui_windows_common::{CustomButton, MessageBox};
