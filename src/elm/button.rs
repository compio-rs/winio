use crate::{Component, ComponentSender, Point, Size, Window, ui};

pub struct Button {
    widget: ui::Button,
}

impl Button {
    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.widget.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.widget.set_size(v)
    }

    pub fn text(&self) -> String {
        self.widget.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }
}

#[non_exhaustive]
pub enum ButtonEvent {
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
