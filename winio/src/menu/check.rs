use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource};
use winio_handle::BorrowedMenuContainer;
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A menu item whose checked state is switched by the user, and which is
/// independent of the other check menu items around it.
#[derive(Debug)]
pub struct CheckMenuItem {
    inner: sys::CheckMenuItem,
    checked_prop: PropSource<bool>,
}

impl Failable for CheckMenuItem {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for CheckMenuItem {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
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
    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        if checked != self.is_checked()? {
            self.inner.set_checked(checked)?;
            self.checked_prop.notify(checked);
        }
        Ok(())
    }

    /// Property for [`CheckMenuItem::is_checked`].
    pub fn checked_prop(&mut self) -> Result<Prop<'_, bool>> {
        Ok(self.checked_prop.as_prop(self.is_checked()?))
    }
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
    /// The checked state has been changed by user click.
    #[doc(hidden)]
    ChangeInputChecked,
    /// Set the checked state.
    SetChecked(bool),
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
        let checked_prop = PropSource::new();
        Ok(Self {
            inner,
            checked_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.post(CheckMenuItemMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            CheckMenuItemMessage::ChangeInputChecked => {
                let checked = self.is_checked()?;
                self.checked_prop.notify(checked);
                sender.output(CheckMenuItemEvent::Click);
                Ok(false)
            }
            CheckMenuItemMessage::SetChecked(checked) => {
                self.set_checked(checked)?;
                Ok(true)
            }
            CheckMenuItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_item!(CheckMenuItem, inner);
