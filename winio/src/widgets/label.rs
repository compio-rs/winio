use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, HAlign, Layoutable, Point, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple single-line label.
#[derive(Debug)]
pub struct Label {
    widget: sys::Label,
}

impl Failable for Label {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Label {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for Label {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Label {
    /// The horizontal alignment.
    pub fn halign(&self) -> Result<HAlign>;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    /// If the label background is transparent.
    #[cfg(all(windows, feature = "win32"))]
    pub fn is_transparent(&self) -> Result<bool>;

    /// Set if the label background is transparent.
    #[cfg(all(windows, feature = "win32"))]
    pub fn set_transparent(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Label {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Label {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Label {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Label`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LabelEvent {}

/// Messages of [`Label`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LabelMessage {}

impl Component for Label {
    type Error = Error;
    type Event = LabelEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = LabelMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Label::new(init)?;
        Ok(Self { widget })
    }
}

winio_handle::impl_as_widget!(Label, widget);
