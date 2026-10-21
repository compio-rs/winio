use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource, Radio, RadioEvent};
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys::{self, Error, Result},
    tool_bar::ToolBar,
    ui::Image,
};

/// A radio button in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarRadioButton {
    inner: sys::ToolBarRadioButton,
    checked_prop: PropSource<bool>,
}

impl Failable for ToolBarRadioButton {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for ToolBarRadioButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for ToolBarRadioButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl Radio for ToolBarRadioButton {
    fn is_checked(&self) -> Result<bool>;

    fn set_checked(&mut self, checked: bool) -> Result<()> {
        if checked != self.is_checked()? {
            self.inner.set_checked(checked)?;
            self.checked_prop.notify(checked);
        }
        Ok(())
    }
}

impl ToolBarRadioButton {
    /// Property for [`ToolBarRadioButton::is_checked`].
    pub fn checked_prop(&mut self) -> Result<Prop<'_, bool>> {
        Ok(self.checked_prop.as_prop(self.is_checked()?))
    }

    /// Set the icon of the radio button.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.inner.set_icon(icon.map(|icon| &icon.0))
    }
}

/// Events of [`ToolBarRadioButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarRadioButtonEvent {
    /// The radio button has been clicked.
    Click,
}

/// Messages of [`ToolBarRadioButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarRadioButtonMessage {
    /// The checked state has been changed by user click.
    #[doc(hidden)]
    ChangeInputChecked,
    /// Set the checked state.
    SetChecked(bool),
    /// Set the text.
    SetText(String),
}

impl Component for ToolBarRadioButton {
    type Error = Error;
    type Event = ToolBarRadioButtonEvent;
    type Init<'a> = &'a ToolBar;
    type Message = ToolBarRadioButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBarRadioButton::new(init.as_sys())?;
        let checked_prop = PropSource::new();
        Ok(Self {
            inner,
            checked_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.post(ToolBarRadioButtonMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ToolBarRadioButtonMessage::ChangeInputChecked => {
                let checked = self.is_checked()?;
                self.checked_prop.notify(checked);
                sender.output(ToolBarRadioButtonEvent::Click);
                Ok(false)
            }
            ToolBarRadioButtonMessage::SetChecked(checked) => {
                self.set_checked(checked)?;
                Ok(true)
            }
            ToolBarRadioButtonMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

impl RadioEvent for ToolBarRadioButtonEvent {
    fn is_click(&self) -> bool {
        matches!(self, Self::Click)
    }
}
