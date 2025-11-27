use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, Orient, Point, Size, TickPosition, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple button.
#[derive(Debug)]
pub struct Slider {
    widget: sys::Slider,
}

impl Failable for Slider {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Slider {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Slider {
    /// The tick position.
    pub fn tick_pos(&self) -> Result<TickPosition>;

    /// Set the tick position.
    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()>;

    /// The orientation.
    pub fn orient(&self) -> Result<Orient>;

    /// Set the orientation.
    pub fn set_orient(&mut self, v: Orient) -> Result<()>;

    /// Value minimum.
    pub fn minimum(&self) -> Result<usize>;

    /// Set value minimum.
    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    /// Value maximum.
    pub fn maximum(&self) -> Result<usize>;

    /// Set value maximum.
    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    /// The tick frequency.
    pub fn freq(&self) -> Result<usize>;

    /// Set the tick frequency.
    pub fn set_freq(&mut self, v: usize) -> Result<()>;

    /// The position.
    pub fn pos(&self) -> Result<usize>;

    /// Set the position.
    pub fn set_pos(&mut self, v: usize) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Slider {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Slider {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Slider {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Slider`].
#[non_exhaustive]
pub enum SliderEvent {
    /// The position of slider has changed.
    Change,
}

impl Component for Slider {
    type Error = Error;
    type Event = SliderEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Slider::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.output(SliderEvent::Change);
        }
    }
}

winio_handle::impl_as_widget!(Slider, widget);
