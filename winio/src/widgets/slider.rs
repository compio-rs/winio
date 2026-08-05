use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, Prop, PropSinkEvent, PropSinkMessage, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, Orient, Point, Rect, Size, TickPosition, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A slider.
#[derive(Debug)]
pub struct Slider {
    widget: sys::Slider,
    pos_prop: Child<Prop<usize>>,
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
    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        self.pos_prop.set(v);
        Ok(())
    }

    /// Property for [`Slider::pos`].
    pub fn pos_prop(&mut self) -> &mut Prop<usize> {
        &mut self.pos_prop
    }
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

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Slider`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SliderEvent {
    /// The position has been changed.
    Change,
}

/// Messages of [`Slider`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SliderMessage {
    /// No operation.
    Noop,
    /// The position has been changed by user input.
    ChangeInputPos,
    /// The position prop has been changed.
    ChangePropPos,
    /// Set the position.
    SetPos(usize),
    /// Set the rect.
    SetRect(Rect),
    /// Set the minimum.
    SetMinimum(usize),
    /// Set the maximum.
    SetMaximum(usize),
    /// Set the tick frequency.
    SetFreq(usize),
    /// Set the tick position.
    SetTickPos(TickPosition),
    /// Set the orientation.
    SetOrient(Orient),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
}

impl Component for Slider {
    type Error = Error;
    type Event = SliderEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = SliderMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Slider::new(init)?;
        let Ok(pos_prop) = Child::<Prop<usize>>::init(widget.pos()?).await;
        Ok(Self { widget, pos_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_change = async {
            loop {
                self.widget.wait_change().await;
                sender.post(SliderMessage::ChangeInputPos);
            }
        };
        let fut_props = async {
            start! {
                sender, default: SliderMessage::Noop,
                self.pos_prop => { PropSinkEvent::Changed => SliderMessage::ChangePropPos },
            }
        };
        futures_util::future::join(fut_change, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.pos_prop.update().await;
        Ok(r0)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            SliderMessage::Noop => Ok(false),
            SliderMessage::ChangeInputPos => {
                let pos = self.widget.pos()?;
                self.pos_prop.post(PropSinkMessage::Set(pos));
                sender.output(SliderEvent::Change);
                Ok(false)
            }
            SliderMessage::ChangePropPos => {
                let current = self.widget.pos()?;
                let prop_val = self.pos_prop.get();
                if current != *prop_val {
                    self.widget.set_pos(*prop_val)?;
                }
                Ok(true)
            }
            SliderMessage::SetPos(pos) => {
                self.pos_prop.set(pos);
                Ok(true)
            }
            SliderMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            SliderMessage::SetMinimum(minimum) => {
                self.set_minimum(minimum)?;
                Ok(false)
            }
            SliderMessage::SetMaximum(maximum) => {
                self.set_maximum(maximum)?;
                Ok(false)
            }
            SliderMessage::SetFreq(freq) => {
                self.set_freq(freq)?;
                Ok(true)
            }
            SliderMessage::SetTickPos(tick_pos) => {
                self.set_tick_pos(tick_pos)?;
                Ok(true)
            }
            SliderMessage::SetOrient(orient) => {
                self.set_orient(orient)?;
                Ok(true)
            }
            SliderMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            SliderMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            SliderMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(Slider, widget);
