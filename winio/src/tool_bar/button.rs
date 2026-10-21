use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys::{self, Error, Result},
    tool_bar::ToolBar,
    ui::Image,
};

/// A plain button in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarButton {
    inner: sys::ToolBarButton,
}

impl Failable for ToolBarButton {
    type Error = Error;
}

#[inherit_methods(from = "self.inner")]
impl Enable for ToolBarButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods(from = "self.inner")]
impl TextWidget for ToolBarButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl ToolBarButton {
    /// Set the icon of the tool bar button.
    ///
    /// If `icon` is `None`, the icon is removed.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.inner.set_icon(icon.map(|icon| &icon.0))
    }
}

/// Events of [`ToolBarButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarButtonEvent {
    /// The tool bar button has been clicked.
    Click,
}

/// Messages of [`ToolBarButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarButtonMessage {
    /// The text has been changed.
    SetText(String),
}

impl Component for ToolBarButton {
    type Error = Error;
    type Event = ToolBarButtonEvent;
    type Init<'a> = &'a ToolBar;
    type Message = ToolBarButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBarButton::new(init.as_sys())?;
        Ok(Self { inner })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
            sender.output(ToolBarButtonEvent::Click);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ToolBarButtonMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarButton, inner);
