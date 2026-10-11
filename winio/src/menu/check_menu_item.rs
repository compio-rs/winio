use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedMenuContainer;
use winio_primitive::{Failable, TextWidget};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A menu item whose checked state is switched by the user, and which is
/// independent of the other check menu items around it.
#[derive(Debug)]
pub struct CheckMenuItem {
    inner: sys::CheckMenuItem,
}

impl Failable for CheckMenuItem {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for CheckMenuItem {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl CheckMenuItem {
    /// Gets the checked state.
    pub fn is_checked(&self) -> Result<bool>;

    /// Sets the checked state.
    pub fn set_checked(&mut self, checked: bool) -> Result<()>;
}

/// Events of [`CheckMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CheckMenuItemEvent {
    /// The menu item has been clicked.
    Click,
}

/// Messages of [`CheckMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CheckMenuItemMessage {
    /// The text has been changed.
    SetText(String),
}

impl Component for CheckMenuItem {
    type Error = Error;
    type Event = CheckMenuItemEvent;
    type Init<'a> = BorrowedMenuContainer<'a>;
    type Message = CheckMenuItemMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::CheckMenuItem::new(init)?;
        Ok(Self { inner })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.output(CheckMenuItemEvent::Click);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            CheckMenuItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_item!(CheckMenuItem, inner);
