use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A progress bar.
#[derive(Debug)]
pub struct Progress {
    widget: sys::Progress,
}

impl Failable for Progress {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Progress {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Progress {
    /// Value minimum.
    pub fn minimum(&self) -> Result<usize>;

    /// Set value minimum.
    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    /// Value maximum.
    pub fn maximum(&self) -> Result<usize>;

    /// Set value maximum.
    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    /// Current position.
    pub fn pos(&self) -> Result<usize>;

    /// Set current position.
    pub fn set_pos(&mut self, pos: usize) -> Result<()>;

    /// Get if the progress bar is in indeterminate state.
    pub fn is_indeterminate(&self) -> Result<bool>;

    /// Set if the progress bar is in indeterminate state.
    pub fn set_indeterminate(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Progress {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Progress {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Progress {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Progress`].
#[non_exhaustive]
pub enum ProgressEvent {}

impl Component for Progress {
    type Event = ProgressEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Progress::new(init)?;
        Ok(Self { widget })
    }
}

winio_handle::impl_as_widget!(Progress, widget);
