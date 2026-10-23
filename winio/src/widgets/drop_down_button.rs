use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
    ui::Image,
};

/// A button that opens a menu when clicked.
#[derive(Debug)]
pub struct DropDownButton {
    widget: sys::DropDownButton,
}

impl Failable for DropDownButton {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for DropDownButton {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for DropDownButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl DropDownButton {
    /// Set the icon of the button.
    ///
    /// If `icon` is `None`, the icon is removed.
    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        self.widget.set_icon(icon.map(|icon| &icon.0))
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for DropDownButton {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for DropDownButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for DropDownButton {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`DropDownButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DropDownButtonEvent {}

/// Messages of [`DropDownButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DropDownButtonMessage {
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
    /// Set the text.
    SetText(String),
}

impl Component for DropDownButton {
    type Error = Error;
    type Event = DropDownButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = DropDownButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::DropDownButton::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        self.widget.start().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            DropDownButtonMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            DropDownButtonMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            DropDownButtonMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            DropDownButtonMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
            DropDownButtonMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(DropDownButton, widget);
winio_handle::impl_as_menu_container!(DropDownButton, widget);
