use std::ops::{Deref, DerefMut};

use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start,
};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple radio box. See [`RadioButtonGroup`] for making selection groups.
#[derive(Debug)]
pub struct RadioButton {
    widget: sys::RadioButton,
    checked_prop: Child<Prop<bool>>,
    text_prop: Child<PropSink<String>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
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
    pub fn set_checked(&mut self, v: bool) -> Result<()>;

    /// Property for [`RadioButton::is_checked`].
    pub fn checked_prop(&mut self) -> &mut Prop<bool> {
        &mut self.checked_prop
    }

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&self) -> &PropSink<String> {
        &self.text_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
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

    fn set_size(&mut self, v: Size) -> Result<()>;

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
    /// No operation.
    Noop,
    /// The checked state has been changed by user click.
    ChangeInputChecked,
    /// The checked prop has been changed.
    ChangePropChecked,
    /// The text has been changed.
    ChangeText,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for RadioButton {
    type Error = Error;
    type Event = RadioButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = RadioButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::RadioButton::new(init)?;
        let Ok(checked_prop) = Child::<Prop<bool>>::init(false).await;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            checked_prop,
            text_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_click = async {
            loop {
                self.widget.wait_click().await;
                sender.post(RadioButtonMessage::ChangeInputChecked);
            }
        };
        let fut_props = async {
            start! {
                sender, default: RadioButtonMessage::Noop,
                self.checked_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangePropChecked },
                self.text_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeText },
                self.enabled_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_click, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.checked_prop.update().await;
        let Ok(r1) = self.text_prop.update().await;
        let Ok(r2) = self.enabled_prop.update().await;
        let Ok(r3) = self.visible_prop.update().await;
        let Ok(r4) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioButtonMessage::Noop => Ok(false),
            RadioButtonMessage::ChangeInputChecked => {
                let checked = self.widget.is_checked()?;
                self.checked_prop.post(PropSinkMessage::Set(checked));
                sender.output(RadioButtonEvent::Click);
                Ok(false)
            }
            RadioButtonMessage::ChangePropChecked => {
                let current = self.widget.is_checked()?;
                let prop_val = self.checked_prop.get();
                if current != *prop_val {
                    self.widget.set_checked(*prop_val)?;
                }
                Ok(true)
            }
            RadioButtonMessage::ChangeText => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            RadioButtonMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            RadioButtonMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            RadioButtonMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(RadioButton, widget);

/// A group of [`RadioButton`]. Only one of them could be checked.
pub struct RadioButtonGroup {
    radios: Vec<Child<RadioButton>>,
}

/// Events of [`RadioButtonGroup`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonGroupEvent {
    /// A radio button has been selected, with its index.
    Click(usize),
}

/// Messages of [`RadioButtonGroup`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonGroupMessage {
    /// No operation.
    Noop,
    /// A radio button has been selected, with its index.
    Click(usize),
}

impl Component for RadioButtonGroup {
    type Error = Error;
    type Event = RadioButtonGroupEvent;
    type Init<'a> = Vec<Child<RadioButton>>;
    type Message = RadioButtonGroupMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        Ok(Self { radios: init })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let futures = self
            .radios
            .iter_mut()
            .enumerate()
            .map(|(i, c)| {
                c.start(
                    sender,
                    move |e| match e {
                        RadioButtonEvent::Click => Some(RadioButtonGroupMessage::Click(i)),
                    },
                    || RadioButtonGroupMessage::Noop,
                )
            })
            .collect::<Vec<_>>();
        futures_util::future::join_all(futures).await;
        std::future::pending().await
    }

    async fn update_children(&mut self) -> Result<bool> {
        futures_util::future::try_join_all(self.radios.iter_mut().map(|c| c.update()))
            .await
            .map(|v| v.into_iter().any(|b| b))
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioButtonGroupMessage::Noop => Ok(false),
            RadioButtonGroupMessage::Click(i) => {
                for (idx, r) in self.radios.iter_mut().enumerate() {
                    r.set_checked(idx == i)?;
                }
                sender.output(RadioButtonGroupEvent::Click(i));
                Ok(false)
            }
        }
    }
}

impl Deref for RadioButtonGroup {
    type Target = Vec<Child<RadioButton>>;

    fn deref(&self) -> &Self::Target {
        &self.radios
    }
}

impl DerefMut for RadioButtonGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.radios
    }
}
