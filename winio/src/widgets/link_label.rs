use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple link label.
#[derive(Debug)]
pub struct LinkLabel {
    widget: sys::LinkLabel,
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

    fn set_size(&mut self, s: Size) -> Result<()>;

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
    /// Set the URI.
    SetUri(String),
    /// Set the transparent state.
    #[cfg(win32)]
    SetTransparent(bool),
}

impl Component for LinkLabel {
    type Error = Error;
    type Event = LinkLabelEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = LinkLabelMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::LinkLabel::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.output(LinkLabelEvent::Click);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            LinkLabelMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            LinkLabelMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            LinkLabelMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            LinkLabelMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
            LinkLabelMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
            LinkLabelMessage::SetUri(uri) => {
                self.set_uri(uri)?;
                Ok(true)
            }
            #[cfg(win32)]
            LinkLabelMessage::SetTransparent(transparent) => {
                self.set_transparent(transparent)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(LinkLabel, widget);
