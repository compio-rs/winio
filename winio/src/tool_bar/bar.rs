use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_primitive::{Failable, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A toolbar docked at the top of a window.
#[derive(Debug)]
pub struct ToolBar {
    inner: sys::ToolBar,
}

impl Failable for ToolBar {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Visible for ToolBar {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, visible: bool) -> Result<()>;
}

impl ToolBar {
    pub(crate) fn as_sys(&self) -> &sys::ToolBar {
        &self.inner
    }
}

/// Events of [`ToolBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarEvent {}

/// Messages of [`ToolBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarMessage {}

impl Component for ToolBar {
    type Error = Error;
    type Event = ToolBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ToolBarMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBar::new(init)?;
        Ok(Self { inner })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {}
    }
}
