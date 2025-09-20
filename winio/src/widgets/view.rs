use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_layout::{Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A simple window.
#[derive(Debug)]
pub struct View {
    widget: sys::View,
}

#[inherit_methods(from = "self.widget")]
impl Visible for View {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for View {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);
}

/// Events of [`View`].
#[non_exhaustive]
pub enum ViewEvent {}

impl Component for View {
    type Event = ViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        Self {
            widget: sys::View::new(init),
        }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(View, widget);
winio_handle::impl_as_container!(View, widget);
