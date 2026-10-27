use compio_log::error;
use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A single-line integer input for the user to type and edit the value.
/// A pair of up and down buttons are provided to increase or decrease the
/// value.
#[derive(Debug)]
pub struct UpDownEdit {
    widget: sys::UpDownEdit,
    value_prop: PropSource<usize>,
}

impl Failable for UpDownEdit {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for UpDownEdit {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl UpDownEdit {
    /// Value minimum.
    pub fn minimum(&self) -> Result<usize>;

    /// Set value minimum.
    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    /// Value maximum.
    pub fn maximum(&self) -> Result<usize>;

    /// Set value maximum.
    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    /// The step size.
    pub fn step(&self) -> Result<usize>;

    /// Set the step size.
    pub fn set_step(&mut self, v: usize) -> Result<()>;

    /// The value.
    pub fn value(&self) -> Result<usize>;

    /// Set the value.
    ///
    /// The current value can be unreadable before the control is initialized,
    /// for example when the Windows up-down control has an empty buddy edit.
    /// In that case the value is still applied.
    pub fn set_value(&mut self, v: usize) -> Result<()> {
        if self.value().is_ok_and(|oldv| v == oldv) {
            return Ok(());
        }
        self.widget.set_value(v)?;
        self.value_prop.notify(v);
        Ok(())
    }

    /// Property for [`UpDownEdit::value`].
    pub fn value_prop(&mut self) -> Result<Prop<'_, usize>> {
        Ok(self.value_prop.as_prop(self.value()?))
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for UpDownEdit {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for UpDownEdit {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for UpDownEdit {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`UpDownEdit`].
#[derive(Debug)]
#[non_exhaustive]
pub enum UpDownEditEvent {
    /// The value has been changed.
    Change,
}

/// Messages of [`UpDownEdit`].
#[derive(Debug)]
#[non_exhaustive]
pub enum UpDownEditMessage {
    /// The value has been changed.
    #[doc(hidden)]
    ChangeInput,
    /// Set the value.
    SetValue(usize),
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
    /// Set the minimum.
    SetMinimum(usize),
    /// Set the maximum.
    SetMaximum(usize),
    /// Set the step.
    SetStep(usize),
}

impl Component for UpDownEdit {
    type Error = Error;
    type Event = UpDownEditEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = UpDownEditMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::UpDownEdit::new(init)?;
        let value_prop = PropSource::new();
        Ok(Self { widget, value_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.post(UpDownEditMessage::ChangeInput);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            UpDownEditMessage::ChangeInput => {
                match self.widget.value() {
                    Ok(value) => {
                        self.value_prop.notify(value);
                        sender.output(UpDownEditEvent::Change);
                    }
                    Err(e) => {
                        error!("Failed to get value from UpDownEdit: {}", e);
                    }
                }
                Ok(false)
            }
            UpDownEditMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            UpDownEditMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            UpDownEditMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            UpDownEditMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
            UpDownEditMessage::SetValue(value) => {
                self.set_value(value)?;
                Ok(true)
            }
            UpDownEditMessage::SetMinimum(minimum) => {
                self.set_minimum(minimum)?;
                Ok(false)
            }
            UpDownEditMessage::SetMaximum(maximum) => {
                self.set_maximum(maximum)?;
                Ok(false)
            }
            UpDownEditMessage::SetStep(step) => {
                self.set_step(step)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(UpDownEdit, widget);
