#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub(crate) mod runtime;
mod ui;

pub use runtime::block_on;
pub use ui::*;
