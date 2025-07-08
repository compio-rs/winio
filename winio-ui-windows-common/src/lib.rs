//! Windows common methods for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg(windows)]

mod accent;
pub use accent::*;

mod filebox;
pub use filebox::*;

mod msgbox;
pub use msgbox::*;

mod monitor;
pub use monitor::*;
