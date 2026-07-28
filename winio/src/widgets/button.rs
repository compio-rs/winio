use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple button.
#[derive(Debug)]
pub struct Button {
    widget: sys::Button,
    text_prop: Child<PropSink<String>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
}

impl Failable for Button {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Button {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for Button {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl Button {
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
impl Visible for Button {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Button {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Button {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Button`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ButtonEvent {
    /// The button has been clicked.
    Click,
}

/// Messages of [`Button`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ButtonMessage {
    /// No operation.
    Noop,
    /// The text has been changed.
    ChangeText,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for Button {
    type Error = Error;
    type Event = ButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Button::new(init)?;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
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
                sender.output(ButtonEvent::Click);
            }
        };
        let fut_props = async {
            start! {
                sender, default: ButtonMessage::Noop,
                self.text_prop => { PropSinkEvent::Changed => ButtonMessage::ChangeText },
                self.enabled_prop => { PropSinkEvent::Changed => ButtonMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => ButtonMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => ButtonMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_click, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.enabled_prop.update().await;
        let Ok(r2) = self.visible_prop.update().await;
        let Ok(r3) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ButtonMessage::Noop => Ok(false),
            ButtonMessage::ChangeText => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            ButtonMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            ButtonMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ButtonMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Button, widget);
