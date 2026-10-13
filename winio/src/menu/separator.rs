use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedMenuContainer;
use winio_primitive::Failable;

use crate::{
    sys,
    sys::{Error, Result},
};

/// A horizontal line that separates the groups of menu items around it.
///
/// It carries no text and cannot be clicked.
#[derive(Debug)]
pub struct SeparatorMenuItem {
    inner: sys::SeparatorMenuItem,
}

impl Failable for SeparatorMenuItem {
    type Error = Error;
}

/// Events of [`SeparatorMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SeparatorMenuItemEvent {}

/// Messages of [`SeparatorMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SeparatorMenuItemMessage {}

impl Component for SeparatorMenuItem {
    type Error = Error;
    type Event = SeparatorMenuItemEvent;
    type Init<'a> = BorrowedMenuContainer<'a>;
    type Message = SeparatorMenuItemMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::SeparatorMenuItem::new(init)?;
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

winio_handle::impl_as_menu_item!(SeparatorMenuItem, inner);
