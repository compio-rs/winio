//! Window handle for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

mod window;
pub use window::*;

mod widget;
pub use widget::*;

mod container;
pub use container::*;
