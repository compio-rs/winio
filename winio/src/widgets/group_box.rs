use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Failable, Layoutable, Point, Rect, Size, TextWidget, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A container that groups other widgets together, typically with a border and
/// a title.
#[derive(Debug)]
pub struct GroupBox {
    widget: sys::GroupBox,
}

impl Failable for GroupBox {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl Visible for GroupBox {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for GroupBox {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for GroupBox {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

/// Events of [`GroupBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum GroupBoxEvent {}

/// Messages of [`GroupBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum GroupBoxMessage {
    /// Set the rect.
    SetRect(Rect),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the title of the group box.
    SetText(String),
}

impl Component for GroupBox {
    type Error = Error;
    type Event = GroupBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = GroupBoxMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::GroupBox::new(init)?;
        Ok(Self { widget })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            GroupBoxMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            GroupBoxMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            GroupBoxMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(GroupBox, widget);
winio_handle::impl_as_container!(GroupBox, widget);
