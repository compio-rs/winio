use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{HAlign, Point, Size};

use crate::sys;

/// A simple single-line label.
#[derive(Debug)]
pub struct Label {
    widget: sys::Label,
}

#[inherit_methods(from = "self.widget")]
impl Label {
    /// The text.
    pub fn text(&self) -> String;

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>);

    /// The horizontal alignment.
    pub fn halign(&self) -> HAlign;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign);

    /// If the label background is transparent.
    #[cfg(all(windows, feature = "win32"))]
    pub fn is_transparent(&self) -> bool;

    /// Set if the label background is transparent.
    #[cfg(all(windows, feature = "win32"))]
    pub fn set_transparent(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Visible for Label {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for Label {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Label {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`Label`].
#[non_exhaustive]
pub enum LabelEvent {}

impl Component for Label {
    type Event = LabelEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::Label::new(init);
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

winio_handle::impl_as_widget!(Label, widget);
