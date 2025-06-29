use inherit_methods_macro::inherit_methods;

use crate::{
    BorrowedWindow, Component, ComponentSender, Enable, Layoutable, Point, Size, Visible, ui,
};

/// A simple check box.
#[derive(Debug)]
pub struct CheckBox {
    widget: ui::CheckBox,
}

#[inherit_methods(from = "self.widget")]
impl CheckBox {
    /// The text.
    pub fn text(&self) -> String;

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>);

    /// If the box is checked.
    pub fn is_checked(&self) -> bool;

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Visible for CheckBox {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for CheckBox {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for CheckBox {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`CheckBox`].
#[non_exhaustive]
pub enum CheckBoxEvent {
    /// The check box has been clicked.
    Click,
}

impl Component for CheckBox {
    type Event = CheckBoxEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::CheckBox::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.output(CheckBoxEvent::Click);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
