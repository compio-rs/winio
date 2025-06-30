//! A single-threaded asynchronous GUI runtime, based on [`compio`] and ELM
//! design.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

#[doc(no_inline)]
pub use compio;
#[doc(inline)]
pub use winio_elm::{
    Child, Component, ComponentSender, ObservableVec, ObservableVecEvent, init, start,
};
#[doc(inline)]
pub use winio_handle::*;
#[doc(inline)]
pub use winio_layout::{Enable, Grid, Layoutable, StackPanel, Visible, layout};
#[doc(inline)]
pub use winio_primitive::*;

pub(crate) mod ui;
pub use ui::export::*;

mod elm;
pub use elm::*;
