//! Primitive types for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

mod drawing;
pub use drawing::*;

mod monitor;
pub use monitor::*;

mod canvas;
pub use canvas::*;

mod msgbox;
pub use msgbox::*;
