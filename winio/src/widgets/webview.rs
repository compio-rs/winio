use std::fmt::Debug;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::{AsRawWidget, AsWindow, RawWidget};
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

struct LazyInitParams {
    visible: bool,
    enabled: bool,
    loc: Point,
    size: Size,
    source: String,
}

impl Default for LazyInitParams {
    fn default() -> Self {
        Self {
            visible: true,
            enabled: true,
            loc: Default::default(),
            size: Default::default(),
            source: Default::default(),
        }
    }
}

enum WebViewInner {
    Params(LazyInitParams),
    Widget(sys::WebView),
}

impl Debug for WebViewInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebViewInner").finish_non_exhaustive()
    }
}

impl WebViewInner {
    async fn init(&mut self, parent: impl AsWindow) {
        match self {
            Self::Params(p) => {
                let mut w = sys::WebView::new(parent).await;
                w.set_visible(p.visible);
                w.set_enabled(p.enabled);
                w.set_loc(p.loc);
                w.set_size(p.size);
                w.set_source(&p.source);
                *self = Self::Widget(w);
            }
            Self::Widget(_) => {}
        }
    }
}

impl WebViewInner {
    fn is_visible(&self) -> bool {
        match self {
            Self::Params(p) => p.visible,
            Self::Widget(w) => w.is_visible(),
        }
    }

    fn set_visible(&mut self, v: bool) {
        match self {
            Self::Params(p) => p.visible = v,
            Self::Widget(w) => w.set_visible(v),
        }
    }

    fn is_enabled(&self) -> bool {
        match self {
            Self::Params(p) => p.enabled,
            Self::Widget(w) => w.is_enabled(),
        }
    }

    fn set_enabled(&mut self, v: bool) {
        match self {
            Self::Params(p) => p.enabled = v,
            Self::Widget(w) => w.set_enabled(v),
        }
    }

    fn loc(&self) -> Point {
        match self {
            Self::Params(p) => p.loc,
            Self::Widget(w) => w.loc(),
        }
    }

    fn set_loc(&mut self, v: Point) {
        match self {
            Self::Params(p) => p.loc = v,
            Self::Widget(w) => w.set_loc(v),
        }
    }

    fn size(&self) -> Size {
        match self {
            Self::Params(p) => p.size,
            Self::Widget(w) => w.size(),
        }
    }

    fn set_size(&mut self, v: Size) {
        match self {
            Self::Params(p) => p.size = v,
            Self::Widget(w) => w.set_size(v),
        }
    }

    fn preferred_size(&self) -> Size {
        match self {
            Self::Params(_) => Size::zero(),
            Self::Widget(w) => w.preferred_size(),
        }
    }

    pub fn source(&self) -> String {
        match self {
            Self::Params(p) => p.source.clone(),
            Self::Widget(w) => w.source(),
        }
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) {
        match self {
            Self::Params(p) => p.source = s.as_ref().into(),
            Self::Widget(w) => w.set_source(s),
        }
    }

    pub fn can_go_forward(&self) -> bool {
        match self {
            Self::Params(_) => false,
            Self::Widget(w) => w.can_go_forward(),
        }
    }

    pub fn go_forward(&mut self) {
        match self {
            Self::Params(_) => unreachable!("cannot go forward before initialized"),
            Self::Widget(w) => w.go_forward(),
        }
    }

    pub fn can_go_back(&self) -> bool {
        match self {
            Self::Params(_) => false,
            Self::Widget(w) => w.can_go_back(),
        }
    }

    pub fn go_back(&mut self) {
        match self {
            Self::Params(_) => unreachable!("cannot go back before initialized"),
            Self::Widget(w) => w.go_back(),
        }
    }

    pub async fn wait_navigate(&self) {
        match self {
            Self::Params(_) => std::future::pending().await,
            Self::Widget(w) => w.wait_navigate().await,
        }
    }
}

impl AsRawWidget for WebViewInner {
    fn as_raw_widget(&self) -> RawWidget {
        match self {
            Self::Params(_) => unreachable!("cannot get raw widget before initialized"),
            Self::Widget(w) => w.as_raw_widget(),
        }
    }
}

/// A web view.
#[derive(Debug)]
pub struct WebView {
    widget: WebViewInner,
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    /// Initialize the web engine asynchronously.
    pub async fn init(&mut self, parent: impl AsWindow) {
        self.widget.init(parent).await;
    }

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
    type Init<'a> = ();
    type Message = ();

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = WebViewInner::Params(LazyInitParams::default());
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
