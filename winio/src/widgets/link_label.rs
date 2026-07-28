use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple link label.
#[derive(Debug)]
pub struct LinkLabel {
    widget: sys::LinkLabel,
    text_prop: Child<PropSink<String>>,
    uri_prop: Child<PropSink<String>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
    #[cfg(win32)]
    transparent_prop: Child<PropSink<bool>>,
}

impl Failable for LinkLabel {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl LinkLabel {
    /// The URI of the link.
    pub fn uri(&self) -> Result<String>;

    /// Set the URI of the link to navigate. If the URI is not empty, no `Click`
    /// event will be triggered when the link label is clicked, and the system
    /// will try to open the link.
    ///
    /// There is no validation or sanitization for the URI, so be careful when
    /// setting it. This could potentially be exploited with malicious URIs.
    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()>;

    /// If the label background is transparent.
    #[cfg(win32)]
    pub fn is_transparent(&self) -> Result<bool>;

    /// Set if the label background is transparent.
    #[cfg(win32)]
    pub fn set_transparent(&mut self, v: bool) -> Result<()>;

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&self) -> &PropSink<String> {
        &self.text_prop
    }

    /// Property for [`LinkLabel::uri`].
    pub fn uri_prop(&self) -> &PropSink<String> {
        &self.uri_prop
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

    #[cfg(win32)]
    #[allow(dead_code)]
    /// Property for [`LinkLabel::is_transparent`].
    pub fn transparent_prop(&self) -> &PropSink<bool> {
        &self.transparent_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for LinkLabel {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for LinkLabel {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for LinkLabel {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for LinkLabel {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for LinkLabel {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`LinkLabel`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LinkLabelEvent {
    /// The link label has been clicked.
    /// Note that this event is not triggered if `uri` is not empty.
    Click,
}

/// Messages of [`LinkLabel`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LinkLabelMessage {
    /// No operation.
    Noop,
    /// The text has been changed.
    ChangeText,
    /// The uri has been changed.
    ChangeUri,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
    #[cfg(win32)]
    /// The transparent state has been changed.
    ChangeTransparent,
}

impl Component for LinkLabel {
    type Error = Error;
    type Event = LinkLabelEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = LinkLabelMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::LinkLabel::new(init)?;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(uri_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        #[cfg(win32)]
        let Ok(transparent_prop) = Child::<PropSink<bool>>::init(false).await;
        Ok(Self {
            widget,
            text_prop,
            uri_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
            #[cfg(win32)]
            transparent_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_click = async {
            loop {
                self.widget.wait_click().await;
                sender.output(LinkLabelEvent::Click);
            }
        };
        let fut_props = async {
            #[cfg(win32)]
            {
                start! {
                    sender, default: LinkLabelMessage::Noop,
                    self.text_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeText },
                    self.uri_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeUri },
                    self.enabled_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeEnabled },
                    self.visible_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeVisible },
                    self.tooltip_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeTooltip },
                    self.transparent_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeTransparent },
                }
            }
            #[cfg(not(win32))]
            {
                start! {
                    sender, default: LinkLabelMessage::Noop,
                    self.text_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeText },
                    self.uri_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeUri },
                    self.enabled_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeEnabled },
                    self.visible_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeVisible },
                    self.tooltip_prop => { PropSinkEvent::Changed => LinkLabelMessage::ChangeTooltip },
                }
            }
        };
        futures_util::future::join(fut_click, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.uri_prop.update().await;
        let Ok(r2) = self.enabled_prop.update().await;
        let Ok(r3) = self.visible_prop.update().await;
        let Ok(r4) = self.tooltip_prop.update().await;
        #[cfg(win32)]
        let Ok(r5) = self.transparent_prop.update().await;
        #[cfg(not(win32))]
        let r5 = false;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            LinkLabelMessage::Noop => Ok(false),
            LinkLabelMessage::ChangeText => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            LinkLabelMessage::ChangeUri => {
                self.widget.set_uri(self.uri_prop.get())?;
                Ok(true)
            }
            LinkLabelMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            LinkLabelMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            LinkLabelMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
            #[cfg(win32)]
            LinkLabelMessage::ChangeTransparent => {
                self.widget.set_transparent(**self.transparent_prop)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(LinkLabel, widget);
