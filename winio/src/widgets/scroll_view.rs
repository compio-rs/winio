use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A scroll view that can contain other widgets and provide scrolling.
/// functionality.
#[derive(Debug)]
pub struct ScrollView {
    widget: sys::ScrollView,
}

impl Failable for ScrollView {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ScrollView {
    /// Get if the horizontal scroll bar is visible.
    pub fn hscroll(&self) -> Result<bool>;

    /// Set if the horizontal scroll bar is visible.
    pub fn set_hscroll(&mut self, v: bool) -> Result<()>;

    /// Get if the vertical scroll bar is visible.
    pub fn vscroll(&self) -> Result<bool>;

    /// Set if the vertical scroll bar is visible.
    pub fn set_vscroll(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for ScrollView {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for ScrollView {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ScrollView {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

/// Events of [`ScrollView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollViewEvent {}

/// Messages of [`ScrollView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollViewMessage {
    /// No operation.
    Noop,
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the horizontal scroll bar visibility.
    SetHScroll(bool),
    /// Set the vertical scroll bar visibility.
    SetVScroll(bool),
}

impl Component for ScrollView {
    type Error = Error;
    type Event = ScrollViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ScrollViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ScrollView::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        self.widget.start().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ScrollViewMessage::Noop => Ok(false),
            ScrollViewMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            ScrollViewMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            ScrollViewMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            ScrollViewMessage::SetHScroll(hscroll) => {
                self.set_hscroll(hscroll)?;
                Ok(true)
            }
            ScrollViewMessage::SetVScroll(vscroll) => {
                self.set_vscroll(vscroll)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(ScrollView, widget);
winio_handle::impl_as_container!(ScrollView, widget);
