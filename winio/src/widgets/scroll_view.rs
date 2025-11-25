use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, Visible};

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

    fn set_size(&mut self, v: Size) -> Result<()>;
}

/// Events of [`ScrollView`].
#[non_exhaustive]
pub enum ScrollViewEvent {}

impl Component for ScrollView {
    type Event = ScrollViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ScrollView::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        self.widget.start().await
    }
}

winio_handle::impl_as_widget!(ScrollView, widget);
winio_handle::impl_as_container!(ScrollView, widget);
