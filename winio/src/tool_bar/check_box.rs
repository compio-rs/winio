use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource};
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys::{self, Error, Result},
    tool_bar::ToolBar,
    ui::Image,
};

/// A check box in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarCheckBox {
    inner: sys::ToolBarCheckBox,
    checked_prop: PropSource<bool>,
}

impl Failable for ToolBarCheckBox {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for ToolBarCheckBox {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for ToolBarCheckBox {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl ToolBarCheckBox {
    /// If the check box is checked.
    pub fn is_checked(&self) -> Result<bool> {
        self.inner.is_checked()
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        if checked != self.is_checked()? {
            self.inner.set_checked(checked)?;
            self.checked_prop.notify(checked);
        }
        Ok(())
    }

    /// Property for [`ToolBarCheckBox::is_checked`].
    pub fn checked_prop(&mut self) -> Result<Prop<'_, bool>> {
        Ok(self.checked_prop.as_prop(self.is_checked()?))
    }

    /// Set the icon of the check box.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.inner.set_icon(icon.map(|icon| &icon.0))
    }
}

/// Events of [`ToolBarCheckBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarCheckBoxEvent {
    /// The check box has been clicked.
    Click,
}

/// Messages of [`ToolBarCheckBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarCheckBoxMessage {
    /// The checked state has been changed by user click.
    #[doc(hidden)]
    ChangeInputChecked,
    /// Set the checked state.
    SetChecked(bool),
    /// Set the text.
    SetText(String),
}

impl Component for ToolBarCheckBox {
    type Error = Error;
    type Event = ToolBarCheckBoxEvent;
    type Init<'a> = &'a ToolBar;
    type Message = ToolBarCheckBoxMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBarCheckBox::new(init.as_sys())?;
        let checked_prop = PropSource::new();
        Ok(Self {
            inner,
            checked_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.post(ToolBarCheckBoxMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ToolBarCheckBoxMessage::ChangeInputChecked => {
                let checked = self.is_checked()?;
                self.checked_prop.notify(checked);
                sender.output(ToolBarCheckBoxEvent::Click);
                Ok(false)
            }
            ToolBarCheckBoxMessage::SetChecked(checked) => {
                self.set_checked(checked)?;
                Ok(true)
            }
            ToolBarCheckBoxMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarCheckBox, inner);
