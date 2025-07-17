use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A progress bar.
#[derive(Debug)]
pub struct Progress {
    widget: sys::Progress,
}

#[inherit_methods(from = "self.widget")]
impl Progress {
    /// Value range.
    pub fn range(&self) -> (usize, usize);

    /// Set the value range.
    pub fn set_range(&mut self, min: usize, max: usize);

    /// Current position.
    pub fn pos(&self) -> usize;

    /// Set current position.
    pub fn set_pos(&mut self, pos: usize);

    /// Get if the progress bar is in indeterminate state.
    pub fn is_indeterminate(&self) -> bool;

    /// Set if the progress bar is in indeterminate state.
    pub fn set_indeterminate(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Visible for Progress {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for Progress {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Progress {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`Progress`].
#[non_exhaustive]
pub enum ProgressEvent {}

impl Component for Progress {
    type Event = ProgressEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::Progress::new(init);
        Self { widget }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(Progress, widget);
