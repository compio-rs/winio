use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
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
    visible_prop: Child<PropSink<bool>>,
    rect_prop: Child<PropSink<Rect>>,
}

impl Failable for View {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl Visible for View {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

impl View {
    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`Layoutable::rect`].
    pub fn rect_prop(&self) -> &PropSink<Rect> {
        &self.rect_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for View {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(p, rect.size));
        Ok(())
    }

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(rect.origin, s));
        Ok(())
    }
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
    /// The visible state has been changed.
    ChangeVisible,
    /// The rect has been changed.
    ChangeRect,
}

impl Component for View {
    type Error = Error;
    type Event = ViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::View::new(init)?;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let loc = widget.loc()?;
        let size = widget.size()?;
        let rect = Rect::new(loc, size);
        let Ok(rect_prop) = Child::<PropSink<Rect>>::init(rect).await;
        Ok(Self {
            widget,
            visible_prop,
            rect_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: ViewMessage::Noop,
            self.visible_prop => { PropSinkEvent::Changed => ViewMessage::ChangeVisible },
            self.rect_prop => { PropSinkEvent::Changed => ViewMessage::ChangeRect },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.visible_prop.update().await;
        let Ok(r1) = self.rect_prop.update().await;
        Ok(r0 || r1)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ViewMessage::Noop => Ok(false),
            ViewMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ViewMessage::ChangeRect => {
                let rect = *self.rect_prop.get();
                self.widget.set_loc(rect.origin)?;
                self.widget.set_size(rect.size)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(View, widget);
winio_handle::impl_as_container!(View, widget);
