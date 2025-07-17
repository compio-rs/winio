use std::ops::{Deref, DerefMut};

use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, start};
use winio_handle::{AsRawWidget, AsWidget, BorrowedWidget, RawWidget};
use winio_layout::Layoutable;
use winio_primitive::{Point, Rect, Size};

use crate::sys;

/// Tool tip helper for widgets.
pub struct ToolTip<T: Component + AsWidget> {
    widget: sys::ToolTip<Child<T>>,
}

#[inherit_methods(from = "self.widget")]
impl<T: Component + AsWidget> ToolTip<T> {
    /// The tool tip text.
    pub fn tooltip(&self) -> String;

    /// Set the tool tip text.
    pub fn set_tooltip(&mut self, s: impl AsRef<str>);
}

/// Message of [`ToolTip`].
pub enum ToolTipMessage<T: Component> {
    /// Noop message. It does nothing.
    Noop,
    /// Message of the inner widget.
    Message(T::Message),
    /// Event of the inner widget.
    Event(T::Event),
}

impl<T: Component + AsWidget> Component for ToolTip<T> {
    type Event = T::Event;
    type Init<'a> = T::Init<'a>;
    type Message = ToolTipMessage<T>;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::ToolTip::new(Child::init(init));
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {sender, default: ToolTipMessage::Noop,
            self.widget => {
                |e| Some(ToolTipMessage::Event(e))
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        let mut need_render = self.widget.update().await;
        match message {
            ToolTipMessage::Noop => {}
            ToolTipMessage::Message(m) => {
                need_render |= self.widget.emit(m).await;
            }
            ToolTipMessage::Event(e) => {
                sender.output(e);
            }
        }
        need_render
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.widget.render();
    }
}

#[inherit_methods(from = "self.widget")]
impl<T: Component + AsWidget + Layoutable> Layoutable for ToolTip<T> {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, s: Size);

    fn rect(&self) -> Rect;

    fn set_rect(&mut self, r: Rect);

    fn preferred_size(&self) -> Size;

    fn min_size(&self) -> Size;
}

impl<T: AsRawWidget + Component + AsWidget> AsRawWidget for ToolTip<T> {
    fn as_raw_widget(&self) -> RawWidget {
        self.widget.as_raw_widget()
    }
}

impl<T: AsWidget + Component + AsWidget> AsWidget for ToolTip<T> {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        self.widget.as_widget()
    }
}

impl<T: Component + AsWidget> Deref for ToolTip<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<T: Component + AsWidget> DerefMut for ToolTip<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
