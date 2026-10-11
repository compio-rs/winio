use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedMenuContainer;
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A menu item that opens a submenu when the user selects it.
///
/// It is both a menu item of its parent menu and a menu container for its own
/// items, so it can be nested to build an arbitrary menu tree.
#[derive(Debug)]
pub struct PopupMenuItem {
    inner: sys::PopupMenuItem,
}

impl Failable for PopupMenuItem {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for PopupMenuItem {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for PopupMenuItem {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

/// Events of [`PopupMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PopupMenuItemEvent {}

/// Messages of [`PopupMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PopupMenuItemMessage {
    /// Set the text.
    SetText(String),
}

impl Component for PopupMenuItem {
    type Error = Error;
    type Event = PopupMenuItemEvent;
    type Init<'a> = BorrowedMenuContainer<'a>;
    type Message = PopupMenuItemMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::PopupMenuItem::new(init)?;
        Ok(Self { inner })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            PopupMenuItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_container!(PopupMenuItem, inner);
winio_handle::impl_as_menu_item!(PopupMenuItem, inner);
