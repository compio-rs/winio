//! Primitive types for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod traits;
pub use traits::*;

mod drawing;
pub use drawing::*;

mod monitor;
pub use monitor::*;

mod canvas;
pub use canvas::*;

mod msgbox;
pub use msgbox::*;
