use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Orient, Point, Rect, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A bar that allows the user to scroll through a range of values.
#[derive(Debug)]
pub struct ScrollBar {
    widget: sys::ScrollBar,
    pos_prop: PropSource<usize>,
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
    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        if v != self.pos()? {
            self.widget.set_pos(v)?;
            self.pos_prop.notify(v);
        }
        Ok(())
    }

    /// Property for [`ScrollBar::pos`].
    pub fn pos_prop(&mut self) -> Result<Prop<'_, usize>> {
        Ok(self.pos_prop.as_prop(self.pos()?))
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

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`ScrollBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollBarEvent {
    /// The position has been changed.
    Change,
}

/// Messages of [`ScrollBar`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollBarMessage {
    /// The position has been changed by user scroll.
    #[doc(hidden)]
    ChangeInputPos,
    /// Set the position.
    SetPos(usize),
    /// Set the rect.
    SetRect(Rect),
    /// Set the minimum.
    SetMinimum(usize),
    /// Set the maximum.
    SetMaximum(usize),
    /// Set the page.
    SetPage(usize),
    /// Set the orientation.
    SetOrient(Orient),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
}

impl Component for ScrollBar {
    type Error = Error;
    type Event = ScrollBarEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ScrollBarMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ScrollBar::new(init)?;
        let pos_prop = PropSource::new();
        Ok(Self { widget, pos_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.post(ScrollBarMessage::ChangeInputPos);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ScrollBarMessage::ChangeInputPos => {
                let pos = self.widget.pos()?;
                self.pos_prop.notify(pos);
                sender.output(ScrollBarEvent::Change);
                Ok(false)
            }
            ScrollBarMessage::SetPos(pos) => {
                self.set_pos(pos)?;
                Ok(true)
            }
            ScrollBarMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            ScrollBarMessage::SetMinimum(minimum) => {
                self.set_minimum(minimum)?;
                Ok(false)
            }
            ScrollBarMessage::SetMaximum(maximum) => {
                self.set_maximum(maximum)?;
                Ok(false)
            }
            ScrollBarMessage::SetPage(page) => {
                self.set_page(page)?;
                Ok(false)
            }
            ScrollBarMessage::SetOrient(orient) => {
                self.set_orient(orient)?;
                Ok(true)
            }
            ScrollBarMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            ScrollBarMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            ScrollBarMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(ScrollBar, widget);
