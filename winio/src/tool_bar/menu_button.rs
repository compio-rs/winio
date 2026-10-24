use winio_elm::{Component, ComponentSender};
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys::{self, Error, Result},
    tool_bar::ToolBar,
    ui::Image,
};

/// A drop-down button in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarMenuButton {
    inner: sys::ToolBarMenuButton,
}

impl Failable for ToolBarMenuButton {
    type Error = Error;
}

#[inherit_methods_macro::inherit_methods(from = "self.inner")]
impl Enable for ToolBarMenuButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods_macro::inherit_methods(from = "self.inner")]
impl TextWidget for ToolBarMenuButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl ToolBarMenuButton {
    /// Set the icon of the drop-down button.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.inner.set_icon(icon.map(|icon| &icon.0))
    }
}

/// Events of [`ToolBarMenuButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarMenuButtonEvent {}

/// Messages of [`ToolBarMenuButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarMenuButtonMessage {}

impl Component for ToolBarMenuButton {
    type Error = Error;
    type Event = ToolBarMenuButtonEvent;
    type Init<'a> = &'a ToolBar;
    type Message = ToolBarMenuButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBarMenuButton::new(init.as_sys())?;
        Ok(Self { inner })
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        self.inner.start().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {}
    }
}

winio_handle::impl_as_menu_container!(ToolBarMenuButton, inner);
winio_handle::impl_as_tool_bar_item!(ToolBarMenuButton, inner);
