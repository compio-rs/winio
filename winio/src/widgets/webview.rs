use std::fmt::Debug;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A web view.
#[derive(Debug)]
pub struct WebView {
    widget: sys::WebView,
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    /// The current source URL.
    pub fn source(&self) -> String;

    /// Set the source URL to a new one.
    pub fn set_source(&mut self, s: impl AsRef<str>);

    /// Navigate to a new URL.
    pub fn navigate(&mut self, s: impl AsRef<str>) {
        self.set_source(s)
    }

    /// Get if can go forward.
    pub fn can_go_forward(&self) -> bool;

    /// Go forward.
    pub fn go_forward(&mut self);

    /// Get if can go back.
    pub fn can_go_back(&self) -> bool;

    /// Go back.
    pub fn go_back(&mut self);
}

#[inherit_methods(from = "self.widget")]
impl Visible for WebView {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for WebView {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for WebView {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`WebView`].
#[non_exhaustive]
pub enum WebViewEvent {
    /// The webview has been navigated to a new source.
    Navigate,
}

impl Component for WebView {
    type Event = WebViewEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::WebView::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_navigate().await;
            sender.output(WebViewEvent::Navigate);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(WebView, widget);
