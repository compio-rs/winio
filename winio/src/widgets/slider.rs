use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start,
};
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
    pos_prop: Child<Prop<usize>>,
    minimum_prop: Child<PropSink<usize>>,
    maximum_prop: Child<PropSink<usize>>,
    freq_prop: Child<PropSink<usize>>,
    tick_pos_prop: Child<PropSink<TickPosition>>,
    orient_prop: Child<PropSink<Orient>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
}

impl Failable for Slider {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Slider {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.tooltip_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Slider {
    /// The tick position.
    pub fn tick_pos(&self) -> Result<TickPosition>;

    /// Set the tick position.
    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()> {
        self.tick_pos_prop.set(v);
        Ok(())
    }

    /// The orientation.
    pub fn orient(&self) -> Result<Orient>;

    /// Set the orientation.
    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        self.orient_prop.set(v);
        Ok(())
    }

    /// Value minimum.
    pub fn minimum(&self) -> Result<usize>;

    /// Set value minimum.
    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        self.minimum_prop.set(v);
        Ok(())
    }

    /// Value maximum.
    pub fn maximum(&self) -> Result<usize>;

    /// Set value maximum.
    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        self.maximum_prop.set(v);
        Ok(())
    }

    /// The tick frequency.
    pub fn freq(&self) -> Result<usize>;

    /// Set the tick frequency.
    pub fn set_freq(&mut self, v: usize) -> Result<()> {
        self.freq_prop.set(v);
        Ok(())
    }

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

    /// Property for [`Slider::minimum`].
    pub fn minimum_prop(&self) -> &PropSink<usize> {
        &self.minimum_prop
    }

    /// Property for [`Slider::maximum`].
    pub fn maximum_prop(&self) -> &PropSink<usize> {
        &self.maximum_prop
    }

    /// Property for [`Slider::freq`].
    pub fn freq_prop(&self) -> &PropSink<usize> {
        &self.freq_prop
    }

    /// Property for [`Slider::tick_pos`].
    pub fn tick_pos_prop(&self) -> &PropSink<TickPosition> {
        &self.tick_pos_prop
    }

    /// Property for [`Slider::orient`].
    pub fn orient_prop(&self) -> &PropSink<Orient> {
        &self.orient_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for Slider {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Enable for Slider {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.enabled_prop.set(v);
        Ok(())
    }
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
#[derive(Debug)]
#[non_exhaustive]
pub enum SliderEvent {
    /// The position of slider has changed.
    Change,
}

/// Messages of [`Slider`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SliderMessage {
    /// No operation.
    Noop,
    /// The position has been changed by user drag.
    ChangeInputPos,
    /// The position prop has been changed.
    ChangePropPos,
    /// The minimum has been changed.
    ChangeMinimum,
    /// The maximum has been changed.
    ChangeMaximum,
    /// The freq has been changed.
    ChangeFreq,
    /// The tick position has been changed.
    ChangeTickPos,
    /// The orientation has been changed.
    ChangeOrient,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for Slider {
    type Error = Error;
    type Event = SliderEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = SliderMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Slider::new(init)?;
        let Ok(pos_prop) = Child::<Prop<usize>>::init(widget.pos()?).await;
        let Ok(minimum_prop) = Child::<PropSink<usize>>::init(widget.minimum()?).await;
        let Ok(maximum_prop) = Child::<PropSink<usize>>::init(widget.maximum()?).await;
        let Ok(freq_prop) = Child::<PropSink<usize>>::init(widget.freq()?).await;
        let Ok(tick_pos_prop) = Child::<PropSink<TickPosition>>::init(TickPosition::None).await;
        let Ok(orient_prop) = Child::<PropSink<Orient>>::init(widget.orient()?).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            pos_prop,
            minimum_prop,
            maximum_prop,
            freq_prop,
            tick_pos_prop,
            orient_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
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
                self.minimum_prop => { PropSinkEvent::Changed => SliderMessage::ChangeMinimum },
                self.maximum_prop => { PropSinkEvent::Changed => SliderMessage::ChangeMaximum },
                self.freq_prop => { PropSinkEvent::Changed => SliderMessage::ChangeFreq },
                self.tick_pos_prop => { PropSinkEvent::Changed => SliderMessage::ChangeTickPos },
                self.orient_prop => { PropSinkEvent::Changed => SliderMessage::ChangeOrient },
                self.enabled_prop => { PropSinkEvent::Changed => SliderMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => SliderMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => SliderMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_change, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.pos_prop.update().await;
        let Ok(r1) = self.minimum_prop.update().await;
        let Ok(r2) = self.maximum_prop.update().await;
        let Ok(r3) = self.freq_prop.update().await;
        let Ok(r4) = self.tick_pos_prop.update().await;
        let Ok(r5) = self.orient_prop.update().await;
        let Ok(r6) = self.enabled_prop.update().await;
        let Ok(r7) = self.visible_prop.update().await;
        let Ok(r8) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5 || r6 || r7 || r8)
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
            SliderMessage::ChangeMinimum => {
                self.widget.set_minimum(**self.minimum_prop)?;
                Ok(true)
            }
            SliderMessage::ChangeMaximum => {
                self.widget.set_maximum(**self.maximum_prop)?;
                Ok(true)
            }
            SliderMessage::ChangeFreq => {
                self.widget.set_freq(**self.freq_prop)?;
                Ok(true)
            }
            SliderMessage::ChangeTickPos => {
                self.widget.set_tick_pos(*self.tick_pos_prop.get())?;
                Ok(true)
            }
            SliderMessage::ChangeOrient => {
                self.widget.set_orient(*self.orient_prop.get())?;
                Ok(true)
            }
            SliderMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            SliderMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            SliderMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Slider, widget);
