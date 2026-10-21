use winio_elm::{Component, ComponentSender};
use winio_primitive::{Enable, Failable, TextWidget};

use crate::{
    sys::{self, Error, Result},
    tool_bar::ToolBar,
    ui::Image,
};

/// A drop-down button in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarDropDownButton {
    inner: sys::ToolBarDropDownButton,
}

impl Failable for ToolBarDropDownButton {
    type Error = Error;
}

#[inherit_methods_macro::inherit_methods(from = "self.inner")]
impl Enable for ToolBarDropDownButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
}

#[inherit_methods_macro::inherit_methods(from = "self.inner")]
impl TextWidget for ToolBarDropDownButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl ToolBarDropDownButton {
    /// Set the icon of the drop-down button.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.inner.set_icon(icon.map(|icon| &icon.0))
    }
}

/// Events of [`ToolBarDropDownButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarDropDownButtonEvent {}

/// Messages of [`ToolBarDropDownButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolBarDropDownButtonMessage {}

impl Component for ToolBarDropDownButton {
    type Error = Error;
    type Event = ToolBarDropDownButtonEvent;
    type Init<'a> = &'a ToolBar;
    type Message = ToolBarDropDownButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let inner = sys::ToolBarDropDownButton::new(init.as_sys())?;
        Ok(Self { inner })
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        loop {
            self.inner.wait_click().await;
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {}
    }
}

winio_handle::impl_as_menu_container!(ToolBarDropDownButton, inner);
