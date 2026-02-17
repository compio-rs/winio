use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
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
pub enum LinkLabelMessage {}

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
}

winio_handle::impl_as_widget!(LinkLabel, widget);
