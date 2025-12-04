//! Windows common methods for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg(windows)]

pub use windows::core::{Error, Result};

mod accent;
pub use accent::*;

mod filebox;
pub use filebox::*;

mod msgbox;
pub use msgbox::*;

mod monitor;
pub use monitor::*;

mod canvas;
pub use canvas::*;

mod darkmode;
pub use darkmode::*;

mod resource;
pub use resource::*;

mod backdrop;
pub use backdrop::*;

mod runtime;
pub use runtime::*;
