use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedMenuContainer;
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A menu item whose checked state is exclusive among the adjacent radio menu
/// items of the same menu.
#[derive(Debug)]
pub struct RadioMenuItem {
    inner: sys::RadioMenuItem,
}

impl Failable for RadioMenuItem {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for RadioMenuItem {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for RadioMenuItem {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl RadioMenuItem {
    /// Gets the checked state.
    pub fn is_checked(&self) -> Result<bool>;

    /// Sets the checked state.
    pub fn set_checked(&mut self, checked: bool) -> Result<()>;
}

/// Events of [`RadioMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioMenuItemEvent {
    /// The menu item has been clicked.
    Click,
}

/// Messages of [`RadioMenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioMenuItemMessage {
    /// The text has been changed.
    SetText(String),
}

impl Component for RadioMenuItem {
    type Error = Error;
    type Event = RadioMenuItemEvent;
    type Init<'a> = BorrowedMenuContainer<'a>;
    type Message = RadioMenuItemMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::RadioMenuItem::new(init)?;
        Ok(Self { inner })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.output(RadioMenuItemEvent::Click);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioMenuItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, inner);
