use crate::{
    BorrowedWindow, Component, ComponentSender, Enable, HAlign, Layoutable, Point, Size, Visible,
    ui,
};

/// A simple multi-line text input box.
#[derive(Debug)]
pub struct TextBox {
    widget: ui::TextBox,
}

impl TextBox {
    /// The text.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the text.
    ///
    /// Lines are separated with `\n`. You don't need to handle CRLF.
    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }

    /// The horizontal alignment.
    pub fn halign(&self) -> HAlign {
        self.widget.halign()
    }

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) {
        self.widget.set_halign(align);
    }
}

impl Visible for TextBox {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Enable for TextBox {
    fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
    }
}

impl Layoutable for TextBox {
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

/// Events of [`TextBox`].
#[non_exhaustive]
pub enum TextBoxEvent {
    /// The text has been changed.
    Change,
}

impl Component for TextBox {
    type Event = TextBoxEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::TextBox::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        loop {
            self.widget.wait_change().await;
            sender.output(TextBoxEvent::Change);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
