use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_primitive::{Enable, Failable};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A timer without requiring an async runtime.
#[derive(Debug)]
pub struct Timer {
    widget: sys::Timer,
}

impl Failable for Timer {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Timer {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        if v { self.start() } else { self.stop() }
    }
}

#[inherit_methods(from = "self.widget")]
impl Timer {
    /// Starts the timer.
    pub fn start(&mut self) -> Result<()>;

    /// Stops the timer.
    pub fn stop(&mut self) -> Result<()>;
}

/// Events of [`Timer`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TimerEvent {
    /// The timer ticked.
    Tick,
}

/// Messages of [`Timer`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TimerMessage {
    /// Set the enabled state.
    SetEnabled(bool),
}

impl Component for Timer {
    type Error = Error;
    type Event = TimerEvent;
    type Init<'a> = Duration;
    type Message = TimerMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Timer::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait().await;
            sender.output(TimerEvent::Tick);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            TimerMessage::SetEnabled(enabled) => {
                if enabled {
                    self.start()?;
                } else {
                    self.stop()?;
                }
                Ok(false)
            }
        }
    }
}
