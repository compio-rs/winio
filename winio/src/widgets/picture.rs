use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
    ui::Image,
};

/// A widget that displays an image.
#[derive(Debug)]
pub struct Picture {
    widget: sys::Picture,
}

impl Failable for Picture {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Picture {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Picture {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Picture {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Picture {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

impl Picture {
    /// Set the image of the picture.
    ///
    /// If `image` is `None`, the image is removed.
    pub fn set_image(&mut self, image: Option<&Image>) -> Result<()> {
        self.widget.set_image(image.map(|image| &image.0))
    }
}

/// Events of [`Picture`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PictureEvent {}

/// Messages of [`Picture`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PictureMessage {
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
}

impl Component for Picture {
    type Error = Error;
    type Event = PictureEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = PictureMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Picture::new(init)?;
        Ok(Self { widget })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            PictureMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            PictureMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            PictureMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            PictureMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(Picture, widget);
