use std::fmt::Debug;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A web view.
#[derive(Debug)]
pub struct WebView {
    widget: sys::WebView,
}

impl Failable for WebView {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    /// The current source URL.
    pub fn source(&self) -> Result<String>;

    /// Set the source URL to a new one.
    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()>;

    /// Navigate to a new URL.
    pub fn navigate(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.set_source(s)
    }

    /// Set the HTML content directly.
    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()>;

    /// Navigate to HTML content directly.
    pub fn navigate_to_string(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.set_html(s)
    }

    /// Get if can go forward.
    pub fn can_go_forward(&self) -> Result<bool>;

    /// Go forward.
    pub fn go_forward(&mut self) -> Result<()>;

    /// Get if can go back.
    pub fn can_go_back(&self) -> Result<bool>;

    /// Go back.
    pub fn go_back(&mut self) -> Result<()>;

    /// Reload the current page.
    pub fn reload(&mut self) -> Result<()>;

    /// Stop loading the current page.
    pub fn stop(&mut self) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for WebView {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for WebView {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for WebView {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`WebView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WebViewEvent {
    /// The webview is currently navigating to a new source.
    Navigating,
    /// The webview has been navigated to a new source.
    Navigated,
}

/// Messages of [`WebView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WebViewMessage {}

impl Component for WebView {
    type Error = Error;
    type Event = WebViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = WebViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::WebView::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_navigated = async {
            loop {
                self.widget.wait_navigated().await;
                sender.output(WebViewEvent::Navigated);
            }
        };
        let fut_navigating = async {
            loop {
                self.widget.wait_navigating().await;
                sender.output(WebViewEvent::Navigating);
            }
        };
        futures_util::future::join(fut_navigated, fut_navigating)
            .await
            .0
    }
}

winio_handle::impl_as_widget!(WebView, widget);
