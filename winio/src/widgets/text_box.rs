use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_layout::{Enable, Layoutable, TextWidget, ToolTip, Visible};
use winio_primitive::{HAlign, Point, Size};

use crate::sys;

/// A simple multi-line text input box.
#[derive(Debug)]
pub struct TextBox {
    widget: sys::TextBox,
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for TextBox {
    fn tooltip(&self) -> String;

    fn set_tooltip(&mut self, s: impl AsRef<str>);
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for TextBox {
    fn text(&self) -> String;

    fn set_text(&mut self, s: impl AsRef<str>);
}

#[inherit_methods(from = "self.widget")]
impl TextBox {
    /// The horizontal alignment.
    pub fn halign(&self) -> HAlign;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign);
}

#[inherit_methods(from = "self.widget")]
impl Visible for TextBox {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for TextBox {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for TextBox {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;

    fn min_size(&self) -> Size;
}

/// Events of [`TextBox`].
#[non_exhaustive]
pub enum TextBoxEvent {
    /// The text has been changed.
    Change,
}

impl Component for TextBox {
    type Event = TextBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::TextBox::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.output(TextBoxEvent::Change);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(TextBox, widget);
