use crate::{Component, ComponentSender, Point, Size, Window, ui};

/// A simple button.
#[derive(Debug)]
pub struct Button {
    widget: ui::Button,
}

impl Button {
    /// The left top location.
    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    /// Move the location.
    pub fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p)
    }

    /// The size.
    pub fn size(&self) -> Size {
        self.widget.size()
    }

    /// Resize.
    pub fn set_size(&mut self, v: Size) {
        self.widget.set_size(v)
    }

    /// The text.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }
}

/// Events of [`Button`].
#[non_exhaustive]
pub enum ButtonEvent {
    /// The button has been clicked.
    Click,
}

impl Component for Button {
    type Event = ButtonEvent;
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_counter: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Button::new(root);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        loop {
            self.widget.wait_click().await;
            sender.output(ButtonEvent::Click);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
