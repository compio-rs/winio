use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource, Radio, RadioEvent};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A single-choice button. See [`RadioGroup`](winio_elm::RadioGroup) for making
/// selection groups.
#[derive(Debug)]
pub struct RadioButton {
    widget: sys::RadioButton,
    checked_prop: PropSource<bool>,
}

impl Failable for RadioButton {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for RadioButton {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for RadioButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl RadioButton {
    /// If the box is checked.
    pub fn is_checked(&self) -> Result<bool>;

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        if v != self.is_checked()? {
            self.widget.set_checked(v)?;
            self.checked_prop.notify(v);
        }
        Ok(())
    }

    /// Property for [`RadioButton::is_checked`].
    pub fn checked_prop(&mut self) -> Result<Prop<'_, bool>> {
        Ok(self.checked_prop.as_prop(self.is_checked()?))
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for RadioButton {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for RadioButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for RadioButton {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`RadioButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonEvent {
    /// The check box has been clicked.
    Click,
}

/// Messages of [`RadioButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonMessage {
    /// The checked state has been changed by user click.
    #[doc(hidden)]
    ChangeInputChecked,
    /// The checked state is set externally (e.g. from
    /// [`RadioGroup`](winio_elm::RadioGroup)).
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

impl Component for RadioButton {
    type Error = Error;
    type Event = RadioButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = RadioButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::RadioButton::new(init)?;
        let checked_prop = PropSource::new();
        Ok(Self {
            widget,
            checked_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.post(RadioButtonMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioButtonMessage::ChangeInputChecked => {
                let checked = self.widget.is_checked()?;
                self.checked_prop.notify(checked);
                sender.output(RadioButtonEvent::Click);
                Ok(false)
            }
            RadioButtonMessage::SetChecked(v) => {
                self.set_checked(v)?;
                Ok(true)
            }
            RadioButtonMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            RadioButtonMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
            RadioButtonMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            RadioButtonMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            RadioButtonMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(RadioButton, widget);

impl Radio for RadioButton {
    fn is_checked(&self) -> Result<bool> {
        RadioButton::is_checked(self)
    }

    fn set_checked(&mut self, v: bool) -> Result<()> {
        RadioButton::set_checked(self, v)
    }
}

impl RadioEvent for RadioButtonEvent {
    fn is_click(&self) -> bool {
        matches!(self, Self::Click)
    }
}
