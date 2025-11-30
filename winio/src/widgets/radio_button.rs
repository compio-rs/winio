use std::{
    hint::unreachable_unchecked,
    ops::{Deref, DerefMut},
};

use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender};
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
pub enum RadioButtonMessage {}

impl Component for RadioButton {
    type Error = Error;
    type Event = RadioButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = RadioButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::RadioButton::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.output(RadioButtonEvent::Click);
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
                    // `RadioButton` never passes messages.
                    || unsafe { unreachable_unchecked() },
                )
            })
            .collect::<Vec<_>>();
        futures_util::future::join_all(futures).await;
        std::future::pending().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
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
