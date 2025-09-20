use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A scroll view that can contain other widgets and provide scrolling.
/// functionality.
#[derive(Debug)]
pub struct ScrollView {
    widget: sys::ScrollView,
}

#[inherit_methods(from = "self.widget")]
impl ScrollView {
    /// Get if the horizontal scroll bar is visible.
    pub fn hscroll(&self) -> bool;

    /// Set if the horizontal scroll bar is visible.
    pub fn set_hscroll(&mut self, v: bool);

    /// Get if the vertical scroll bar is visible.
    pub fn vscroll(&self) -> bool;

    /// Set if the vertical scroll bar is visible.
    pub fn set_vscroll(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Visible for ScrollView {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for ScrollView {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ScrollView {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);
}

/// Events of [`ScrollView`].
#[non_exhaustive]
pub enum ScrollViewEvent {}

impl Component for ScrollView {
    type Event = ScrollViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::ScrollView::new(init);
        Self { widget }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        self.widget.start().await
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(ScrollView, widget);
winio_handle::impl_as_container!(ScrollView, widget);
