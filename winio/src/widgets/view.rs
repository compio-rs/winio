use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Failable, Layoutable, Point, Size, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple window.
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

    fn set_size(&mut self, v: Size) -> Result<()>;
}

/// Events of [`View`].
#[non_exhaustive]
pub enum ViewEvent {}

impl Component for View {
    type Event = ViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::View::new(init)?;
        Ok(Self { widget })
    }
}

winio_handle::impl_as_widget!(View, widget);
winio_handle::impl_as_container!(View, widget);
