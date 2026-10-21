use winio_elm::{Component, ComponentSender};
use winio_primitive::Failable;

use crate::{
    sys::{self, Error, Result},
    tool_bar::ToolBar,
};

/// A vertical separator in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarSeparator {
    inner: sys::ToolBarSeparator,
}

impl Failable for ToolBarSeparator {
    type Error = Error;
}

/// Events of [`ToolBarSeparator`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarSeparatorEvent {}

/// Messages of [`ToolBarSeparator`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarSeparatorMessage {}

impl Component for ToolBarSeparator {
    type Error = Error;
    type Event = ToolBarSeparatorEvent;
    type Init<'a> = &'a ToolBar;
    type Message = ToolBarSeparatorMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBarSeparator::new(init.as_sys())?;
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

winio_handle::impl_as_tool_bar_item!(ToolBarSeparator, inner);
