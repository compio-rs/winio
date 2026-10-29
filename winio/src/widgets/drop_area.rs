use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Failable, Layoutable, Point, Rect, Size, Visible};

use crate::{
    sys::{self, Error, Result},
    ui::Data,
};

/// An empty container for other widgets that receives dropped data.
#[derive(Debug)]
pub struct DropArea {
    widget: sys::DropArea,
}

impl Failable for DropArea {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl Visible for DropArea {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for DropArea {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

/// Events of [`DropArea`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DropAreaEvent {
    /// The data is dropped.
    Drop(Data),
}

/// Messages of [`DropArea`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DropAreaMessage {
    /// Set the rect.
    SetRect(Rect),
    /// Set the visible state.
    SetVisible(bool),
}

impl Component for DropArea {
    type Error = Error;
    type Event = DropAreaEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = DropAreaMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::DropArea::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            let data = self.widget.wait_drop().await;
            sender.output(DropAreaEvent::Drop(Data::new(data)));
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            DropAreaMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            DropAreaMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(DropArea, widget);
winio_handle::impl_as_container!(DropArea, widget);
