use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedMenuBar;
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A top-level item of a [`MenuBar`](crate::menu::MenuBar), which pops up a
/// menu when the user selects it.
#[derive(Debug)]
pub struct MenuBarItem {
    inner: sys::MenuBarItem,
}

impl Failable for MenuBarItem {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for MenuBarItem {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for MenuBarItem {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

/// Events of [`MenuBarItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MenuBarItemEvent {}

/// Messages of [`MenuBarItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MenuBarItemMessage {
    /// Set the text.
    SetText(String),
}

impl Component for MenuBarItem {
    type Error = Error;
    type Event = MenuBarItemEvent;
    type Init<'a> = BorrowedMenuBar<'a>;
    type Message = MenuBarItemMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::MenuBarItem::new(init)?;
        Ok(Self { inner })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MenuBarItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_container!(MenuBarItem, inner);
