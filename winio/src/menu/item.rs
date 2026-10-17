use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedMenuContainer;
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys,
    sys::{Error, Result},
    ui::Image,
};

/// A plain item of a menu, which triggers an event when the user selects it.
#[derive(Debug)]
pub struct MenuItem {
    inner: sys::MenuItem,
}

impl Failable for MenuItem {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for MenuItem {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for MenuItem {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl MenuItem {
    /// Set the icon of the menu item.
    ///
    /// If `icon` is `None`, the icon is removed.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.inner.set_icon(icon.map(|icon| &icon.0))
    }
}

/// Events of [`MenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MenuItemEvent {
    /// The menu item has been clicked.
    Click,
}

/// Messages of [`MenuItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MenuItemMessage {
    /// The text has been changed.
    SetText(String),
}

impl Component for MenuItem {
    type Error = Error;
    type Event = MenuItemEvent;
    type Init<'a> = BorrowedMenuContainer<'a>;
    type Message = MenuItemMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::MenuItem::new(init)?;
        Ok(Self { inner })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.output(MenuItemEvent::Click);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MenuItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_item!(MenuItem, inner);
