use crate::{
    BorrowedWindow, Component, ComponentSender, Enable, Layoutable, Point, Size, Visible, ui,
};

/// A simple button.
#[derive(Debug)]
pub struct Button {
    widget: ui::Button,
}

impl Button {
    /// The text.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }
}

impl Visible for Button {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Enable for Button {
    fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
    }
}

impl Layoutable for Button {
    fn loc(&self) -> Point {
        self.widget.loc()
    }

    fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p)
    }

    fn size(&self) -> Size {
        self.widget.size()
    }

    fn set_size(&mut self, v: Size) {
        self.widget.set_size(v)
    }

    fn preferred_size(&self) -> Size {
        self.widget.preferred_size()
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
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Button::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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
