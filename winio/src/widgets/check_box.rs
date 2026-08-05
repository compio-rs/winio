use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple check box.
#[derive(Debug)]
pub struct CheckBox {
    widget: sys::CheckBox,
    checked_prop: Prop<bool>,
}

impl Failable for CheckBox {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for CheckBox {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for CheckBox {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl CheckBox {
    /// If the box is checked.
    pub fn is_checked(&self) -> Result<bool>;

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        self.widget.set_checked(v)?;
        self.checked_prop.set(v);
        Ok(())
    }

    /// Property for [`CheckBox::is_checked`].
    pub fn checked_prop(&mut self) -> &mut Prop<bool> {
        &mut self.checked_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for CheckBox {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for CheckBox {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for CheckBox {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`CheckBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CheckBoxEvent {
    /// The check box has been clicked.
    Click,
}

/// Messages of [`CheckBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CheckBoxMessage {
    /// No operation.
    Noop,
    /// The checked state has been changed by user click.
    ChangeInputChecked,
    /// Set the checked state.
    SetChecked(bool),
    /// Set the rect.
    SetRect(Rect),
    /// Set the text.
    SetText(String),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
}

impl Component for CheckBox {
    type Error = Error;
    type Event = CheckBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = CheckBoxMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::CheckBox::new(init)?;
        let mut checked_prop = Prop::new(false);
        checked_prop.bind(sender, CheckBoxMessage::SetChecked);
        Ok(Self { widget, checked_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.post(CheckBoxMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            CheckBoxMessage::Noop => Ok(false),
            CheckBoxMessage::ChangeInputChecked => {
                let checked = self.widget.is_checked()?;
                self.checked_prop.set(checked);
                sender.output(CheckBoxEvent::Click);
                Ok(false)
            }
            CheckBoxMessage::SetChecked(checked) => {
                self.set_checked(checked)?;
                Ok(true)
            }
            CheckBoxMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            CheckBoxMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
            CheckBoxMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            CheckBoxMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            CheckBoxMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(CheckBox, widget);
