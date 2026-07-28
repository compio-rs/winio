use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Orient, Point, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple button.
#[derive(Debug)]
pub struct ScrollBar {
    widget: sys::ScrollBar,
    pos_prop: Child<Prop<usize>>,
    minimum_prop: Child<PropSink<usize>>,
    maximum_prop: Child<PropSink<usize>>,
    page_prop: Child<PropSink<usize>>,
    orient_prop: Child<PropSink<Orient>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
}

impl Failable for ScrollBar {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for ScrollBar {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl ScrollBar {
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

    /// The page size.
    pub fn page(&self) -> Result<usize>;

    /// Set the page size.
    pub fn set_page(&mut self, v: usize) -> Result<()>;

    /// The position.
    pub fn pos(&self) -> Result<usize>;

    /// Set the position.
    pub fn set_pos(&mut self, v: usize) -> Result<()>;

    /// Property for [`ScrollBar::pos`].
    pub fn pos_prop(&mut self) -> &mut Prop<usize> {
        &mut self.pos_prop
    }

    /// Property for [`ScrollBar::minimum`].
    pub fn minimum_prop(&self) -> &PropSink<usize> {
        &self.minimum_prop
    }

    /// Property for [`ScrollBar::maximum`].
    pub fn maximum_prop(&self) -> &PropSink<usize> {
        &self.maximum_prop
    }

    /// Property for [`ScrollBar::page`].
    pub fn page_prop(&self) -> &PropSink<usize> {
        &self.page_prop
    }

    /// Property for [`ScrollBar::orient`].
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
impl Visible for ScrollBar {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for ScrollBar {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ScrollBar {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`ScrollBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollBarEvent {
    /// The position of scroll bar has changed.
    Change,
}

/// Messages of [`ScrollBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollBarMessage {
    /// No operation.
    Noop,
    /// The position has been changed by user scroll.
    ChangeInputPos,
    /// The position prop has been changed.
    ChangePropPos,
    /// The minimum has been changed.
    ChangeMinimum,
    /// The maximum has been changed.
    ChangeMaximum,
    /// The page has been changed.
    ChangePage,
    /// The orientation has been changed.
    ChangeOrient,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for ScrollBar {
    type Error = Error;
    type Event = ScrollBarEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ScrollBarMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ScrollBar::new(init)?;
        let Ok(pos_prop) = Child::<Prop<usize>>::init(0usize).await;
        let Ok(minimum_prop) = Child::<PropSink<usize>>::init(0usize).await;
        let Ok(maximum_prop) = Child::<PropSink<usize>>::init(100usize).await;
        let Ok(page_prop) = Child::<PropSink<usize>>::init(10usize).await;
        let Ok(orient_prop) = Child::<PropSink<Orient>>::init(Orient::Horizontal).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            pos_prop,
            minimum_prop,
            maximum_prop,
            page_prop,
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
                sender.post(ScrollBarMessage::ChangeInputPos);
            }
        };
        let fut_props = async {
            start! {
                sender, default: ScrollBarMessage::Noop,
                self.pos_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangePropPos },
                self.minimum_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangeMinimum },
                self.maximum_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangeMaximum },
                self.page_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangePage },
                self.orient_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangeOrient },
                self.enabled_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => ScrollBarMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_change, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.pos_prop.update().await;
        let Ok(r1) = self.minimum_prop.update().await;
        let Ok(r2) = self.maximum_prop.update().await;
        let Ok(r3) = self.page_prop.update().await;
        let Ok(r4) = self.orient_prop.update().await;
        let Ok(r5) = self.enabled_prop.update().await;
        let Ok(r6) = self.visible_prop.update().await;
        let Ok(r7) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5 || r6 || r7)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ScrollBarMessage::Noop => Ok(false),
            ScrollBarMessage::ChangeInputPos => {
                let pos = self.widget.pos()?;
                self.pos_prop.post(PropSinkMessage::Set(pos));
                sender.output(ScrollBarEvent::Change);
                Ok(false)
            }
            ScrollBarMessage::ChangePropPos => {
                let current = self.widget.pos()?;
                let prop_val = self.pos_prop.get();
                if current != *prop_val {
                    self.widget.set_pos(*prop_val)?;
                }
                Ok(true)
            }
            ScrollBarMessage::ChangeMinimum => {
                self.widget.set_minimum(**self.minimum_prop)?;
                Ok(true)
            }
            ScrollBarMessage::ChangeMaximum => {
                self.widget.set_maximum(**self.maximum_prop)?;
                Ok(true)
            }
            ScrollBarMessage::ChangePage => {
                self.widget.set_page(**self.page_prop)?;
                Ok(true)
            }
            ScrollBarMessage::ChangeOrient => {
                self.widget.set_orient(*self.orient_prop.get())?;
                Ok(true)
            }
            ScrollBarMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            ScrollBarMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ScrollBarMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(ScrollBar, widget);
