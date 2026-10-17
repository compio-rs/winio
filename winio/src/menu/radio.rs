use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource, Radio, RadioEvent};
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
    checked_prop: PropSource<bool>,
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
    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        if checked != self.is_checked()? {
            self.inner.set_checked(checked)?;
            self.checked_prop.notify(checked);
        }
        Ok(())
    }

    /// Property for [`RadioMenuItem::is_checked`].
    pub fn checked_prop(&mut self) -> Result<Prop<'_, bool>> {
        Ok(self.checked_prop.as_prop(self.is_checked()?))
    }
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
    /// The checked state has been changed by user click.
    #[doc(hidden)]
    ChangeInputChecked,
    /// Set the checked state.
    SetChecked(bool),
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
        let checked_prop = PropSource::new();
        Ok(Self {
            inner,
            checked_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.post(RadioMenuItemMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioMenuItemMessage::ChangeInputChecked => {
                let checked = self.is_checked()?;
                self.checked_prop.notify(checked);
                sender.output(RadioMenuItemEvent::Click);
                Ok(false)
            }
            RadioMenuItemMessage::SetChecked(checked) => {
                self.set_checked(checked)?;
                Ok(true)
            }
            RadioMenuItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, inner);

impl Radio for RadioMenuItem {
    fn is_checked(&self) -> Result<bool> {
        RadioMenuItem::is_checked(self)
    }

    fn set_checked(&mut self, v: bool) -> Result<()> {
        RadioMenuItem::set_checked(self, v)
    }
}

impl RadioEvent for RadioMenuItemEvent {
    fn is_click(&self) -> bool {
        matches!(self, Self::Click)
    }
}
