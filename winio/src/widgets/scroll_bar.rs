use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Orient, Point, Size};

use crate::sys;

/// A simple button.
#[derive(Debug)]
pub struct ScrollBar {
    widget: sys::ScrollBar,
}

#[inherit_methods(from = "self.widget")]
impl ScrollBar {
    /// The orientation.
    pub fn orient(&self) -> Orient;

    /// Set the orientation.
    pub fn set_orient(&mut self, v: Orient);

    /// The range of the scroll bar.
    pub fn range(&self) -> (usize, usize);

    /// Set the range of the scroll bar.
    pub fn set_range(&mut self, min: usize, max: usize);

    /// The page size.
    pub fn page(&self) -> usize;

    /// Set the page size.
    pub fn set_page(&mut self, v: usize);

    /// The position.
    pub fn pos(&self) -> usize;

    /// Set the position.
    pub fn set_pos(&mut self, v: usize);
}

#[inherit_methods(from = "self.widget")]
impl Visible for ScrollBar {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for ScrollBar {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ScrollBar {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`ScrollBar`].
#[non_exhaustive]
pub enum ScrollBarEvent {
    /// The position of scroll bar has changed.
    Change,
}

impl Component for ScrollBar {
    type Event = ScrollBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::ScrollBar::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.output(ScrollBarEvent::Change);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(ScrollBar, widget);
