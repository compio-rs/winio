use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Failable, Layoutable, Point, Rect, Size, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple view.
#[derive(Debug)]
pub struct View {
    widget: sys::View,
}

impl Failable for View {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl Visible for View {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for View {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

/// Events of [`View`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ViewEvent {}

/// Messages of [`View`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ViewMessage {
    /// No operation.
    Noop,
    /// Set the rect.
    SetRect(Rect),
    /// Set the visible state.
    SetVisible(bool),
}

impl Component for View {
    type Error = Error;
    type Event = ViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::View::new(init)?;
        Ok(Self { widget })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ViewMessage::Noop => Ok(false),
            ViewMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            ViewMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(View, widget);
winio_handle::impl_as_container!(View, widget);
