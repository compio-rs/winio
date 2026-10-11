use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_primitive::{Failable, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// The menu bar attached to the top of a window.
///
/// It is the root of a menu tree: the items of the menu bar are
/// [`MenuBarItem`](crate::menu::MenuBarItem)s, and each of them pops up a
/// menu of its own.
#[derive(Debug)]
pub struct MenuBar {
    inner: sys::MenuBar,
}

impl Failable for MenuBar {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Visible for MenuBar {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, visible: bool) -> Result<()>;
}

/// Events of [`MenuBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MenuBarEvent {}

/// Messages of [`MenuBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MenuBarMessage {}

impl Component for MenuBar {
    type Error = Error;
    type Event = MenuBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = MenuBarMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::MenuBar::new(init)?;
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

winio_handle::impl_as_menu_bar!(MenuBar, inner);
