use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{HAlign, Point, Size};

use crate::sys;

/// A simple single-line text input box.
#[derive(Debug)]
pub struct Edit {
    widget: sys::Edit,
}

#[inherit_methods(from = "self.widget")]
impl Edit {
    /// The text.
    pub fn text(&self) -> String;

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>);

    /// If the text input is password.
    pub fn is_password(&self) -> bool;

    /// Set if the text input is password.
    pub fn set_password(&mut self, v: bool);

    /// The horizontal alignment.
    pub fn halign(&self) -> HAlign;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign);
}

#[inherit_methods(from = "self.widget")]
impl Visible for Edit {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for Edit {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Edit {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`Edit`].
#[non_exhaustive]
pub enum EditEvent {
    /// The text has been changed.
    Change,
}

impl Component for Edit {
    type Event = EditEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::Edit::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.output(EditEvent::Change);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
