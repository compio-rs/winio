use std::fmt::Debug;

use cookie::Cookie;
use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Rect, Enable, Failable, Layoutable, Point, Size, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A web view.
#[derive(Debug)]
pub struct WebView {
    widget: sys::WebView,
    source_prop: Child<PropSink<String>>,
    visible_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
    rect_prop: Child<PropSink<Rect>>,
}

impl Failable for WebView {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    /// The current source URL.
    pub fn source(&self) -> Result<String>;

    /// Set the source URL to a new one.
    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.source_prop.set(s.as_ref().to_owned());
        Ok(())
    }

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

    /// Get the cookies.
    ///
    /// ## Platform specific
    /// * Android: returns cookies for the current URL.
    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        self.widget.cookies().await
    }

    /// Set a cookie.
    ///
    /// ## Platform specific
    /// * Qt: the method doesn't wait for the cookie to be set.
    /// * Android: sets a cookie for the current URL.
    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        self.widget.set_cookie(c).await
    }

    /// Delete a cookie.
    ///
    /// ## Platform specific
    /// * Qt: the method doesn't wait for the cookie to be deleted.
    /// * Android: deletes a cookie for the current URL.
    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        self.widget.delete_cookie(c).await
    }

    /// Run JavaScript and get the result as a string.
    ///
    /// Be careful when using the returned string, as it may contain different
    /// data on different platforms.
    ///
    /// This method is not a usual `async fn`. It runs the JavaScript code
    /// immediately, and returns a future that waits for the result. This design
    /// allows you to spawn the returned future.
    pub fn run_javascript(
        &mut self,
        js: impl AsRef<str>,
    ) -> Result<impl Future<Output = Result<String>> + 'static> {
        self.widget.run_javascript(js)
    }

    /// Property for [`WebView::source`].
    pub fn source_prop(&self) -> &PropSink<String> {
        &self.source_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Layoutable::rect`].
    pub fn rect_prop(&self) -> &PropSink<Rect> {
        &self.rect_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for WebView {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Enable for WebView {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.enabled_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for WebView {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(p, rect.size));
        Ok(())
    }

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(rect.origin, s));
        Ok(())
    }
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
pub enum WebViewMessage {
    /// No operation.
    Noop,
    /// The source has been changed.
    ChangeSource,
    /// The visible state has been changed.
    ChangeVisible,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The rect has been changed.
    ChangeRect,
}

impl Component for WebView {
    type Error = Error;
    type Event = WebViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = WebViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::WebView::new(init).await?;
        let Ok(source_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let loc = widget.loc()?;
        let size = widget.size()?;
        let rect = Rect::new(loc, size);
        let Ok(rect_prop) = Child::<PropSink<Rect>>::init(rect).await;
        Ok(Self {
            widget,
            source_prop,
            visible_prop,
        rect_prop,
            enabled_prop,
        })
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
        let fut_props = async {
            start! {
                sender, default: WebViewMessage::Noop,
                self.source_prop => { PropSinkEvent::Changed => WebViewMessage::ChangeSource },
                self.visible_prop => { PropSinkEvent::Changed => WebViewMessage::ChangeVisible },
                self.enabled_prop => { PropSinkEvent::Changed => WebViewMessage::ChangeEnabled },
                self.rect_prop => { PropSinkEvent::Changed => WebViewMessage::ChangeRect },
            }
        };
        futures_util::future::join3(fut_navigated, fut_navigating, fut_props)
            .await
            .0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.source_prop.update().await;
        let Ok(r1) = self.visible_prop.update().await;
        let Ok(r2) = self.enabled_prop.update().await;
        let Ok(r3) = self.rect_prop.update().await;
        Ok(r0 || r1 || r2 || r3)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            WebViewMessage::Noop => Ok(false),
            WebViewMessage::ChangeSource => {
                self.widget.set_source(self.source_prop.get())?;
                Ok(true)
            }
            WebViewMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            WebViewMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            WebViewMessage::ChangeRect => {
                let rect = *self.rect_prop.get();
                self.widget.set_loc(rect.origin)?;
                self.widget.set_size(rect.size)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(WebView, widget);
