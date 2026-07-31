use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A progress bar.
#[derive(Debug)]
pub struct Progress {
    widget: sys::Progress,
    pos_prop: Child<PropSink<usize>>,
    minimum_prop: Child<PropSink<usize>>,
    maximum_prop: Child<PropSink<usize>>,
    indeterminate_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
}

impl Failable for Progress {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Progress {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.tooltip_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Progress {
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

    /// Current position.
    pub fn pos(&self) -> Result<usize>;

    /// Set current position.
    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        self.pos_prop.set(pos);
        Ok(())
    }

    /// Get if the progress bar is in indeterminate state.
    pub fn is_indeterminate(&self) -> Result<bool>;

    /// Set if the progress bar is in indeterminate state.
    pub fn set_indeterminate(&mut self, v: bool) -> Result<()> {
        self.indeterminate_prop.set(v);
        Ok(())
    }

    /// Property for [`Progress::pos`].
    pub fn pos_prop(&self) -> &PropSink<usize> {
        &self.pos_prop
    }

    /// Property for [`Progress::minimum`].
    pub fn minimum_prop(&self) -> &PropSink<usize> {
        &self.minimum_prop
    }

    /// Property for [`Progress::maximum`].
    pub fn maximum_prop(&self) -> &PropSink<usize> {
        &self.maximum_prop
    }

    /// Property for [`Progress::is_indeterminate`].
    pub fn indeterminate_prop(&self) -> &PropSink<bool> {
        &self.indeterminate_prop
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
impl Visible for Progress {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Enable for Progress {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.enabled_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Progress {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Progress`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ProgressEvent {}

/// Messages of [`Progress`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ProgressMessage {
    /// No operation.
    Noop,
    /// The position has been changed.
    ChangePos,
    /// The minimum has been changed.
    ChangeMinimum,
    /// The maximum has been changed.
    ChangeMaximum,
    /// The indeterminate state has been changed.
    ChangeIndeterminate,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for Progress {
    type Error = Error;
    type Event = ProgressEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ProgressMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Progress::new(init)?;
        let Ok(pos_prop) = Child::<PropSink<usize>>::init(widget.pos()?).await;
        let Ok(minimum_prop) = Child::<PropSink<usize>>::init(widget.minimum()?).await;
        let Ok(maximum_prop) = Child::<PropSink<usize>>::init(widget.maximum()?).await;
        let Ok(indeterminate_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            pos_prop,
            minimum_prop,
            maximum_prop,
            indeterminate_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: ProgressMessage::Noop,
            self.pos_prop => { PropSinkEvent::Changed => ProgressMessage::ChangePos },
            self.minimum_prop => { PropSinkEvent::Changed => ProgressMessage::ChangeMinimum },
            self.maximum_prop => { PropSinkEvent::Changed => ProgressMessage::ChangeMaximum },
            self.indeterminate_prop => { PropSinkEvent::Changed => ProgressMessage::ChangeIndeterminate },
            self.enabled_prop => { PropSinkEvent::Changed => ProgressMessage::ChangeEnabled },
            self.visible_prop => { PropSinkEvent::Changed => ProgressMessage::ChangeVisible },
            self.tooltip_prop => { PropSinkEvent::Changed => ProgressMessage::ChangeTooltip },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.pos_prop.update().await;
        let Ok(r1) = self.minimum_prop.update().await;
        let Ok(r2) = self.maximum_prop.update().await;
        let Ok(r3) = self.indeterminate_prop.update().await;
        let Ok(r4) = self.enabled_prop.update().await;
        let Ok(r5) = self.visible_prop.update().await;
        let Ok(r6) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5 || r6)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ProgressMessage::Noop => Ok(false),
            ProgressMessage::ChangePos => {
                self.widget.set_pos(**self.pos_prop)?;
                Ok(true)
            }
            ProgressMessage::ChangeMinimum => {
                self.widget.set_minimum(**self.minimum_prop)?;
                Ok(true)
            }
            ProgressMessage::ChangeMaximum => {
                self.widget.set_maximum(**self.maximum_prop)?;
                Ok(true)
            }
            ProgressMessage::ChangeIndeterminate => {
                self.widget.set_indeterminate(**self.indeterminate_prop)?;
                Ok(true)
            }
            ProgressMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            ProgressMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ProgressMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Progress, widget);
