use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Orient, Point, Size};

use crate::sys;

/// A simple button.
#[derive(Debug)]
pub struct Slider {
    widget: sys::Slider,
}

#[inherit_methods(from = "self.widget")]
impl Slider {
    /// The orientation.
    pub fn orient(&self) -> Orient;

    /// Set the orientation.
    pub fn set_orient(&mut self, v: Orient);

    /// The range of the slider.
    pub fn range(&self) -> (usize, usize);

    /// Set the range of the slider.
    pub fn set_range(&mut self, min: usize, max: usize);

    /// The tick frequency.
    pub fn freq(&self) -> usize;

    /// Set the tick frequency.
    pub fn set_freq(&mut self, v: usize);

    /// The position.
    pub fn pos(&self) -> usize;

    /// Set the position.
    pub fn set_pos(&mut self, v: usize);
}

#[inherit_methods(from = "self.widget")]
impl Visible for Slider {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for Slider {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Slider {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`Slider`].
#[non_exhaustive]
pub enum SliderEvent {
    /// The position of slider has changed.
    Change,
}

impl Component for Slider {
    type Event = SliderEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::Slider::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.output(SliderEvent::Change);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(Slider, widget);
