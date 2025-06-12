use crate::{
    BorrowedWindow, Component, ComponentSender, HAlign, Layoutable, Point, Size, Visible, ui,
};

/// A simple single-line text input box.
#[derive(Debug)]
pub struct Edit {
    widget: ui::Edit,
}

impl Edit {
    /// The text.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }

    /// If the text input is password.
    pub fn is_password(&self) -> bool {
        self.widget.is_password()
    }

    /// Set if the text input is password.
    pub fn set_password(&mut self, v: bool) {
        self.widget.set_password(v);
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

impl Visible for Edit {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Layoutable for Edit {
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

/// Events of [`Edit`].
#[non_exhaustive]
pub enum EditEvent {
    /// The text has been changed.
    Change,
}

impl Component for Edit {
    type Event = EditEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Edit::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        loop {
            self.widget.wait_change().await;
            sender.output(EditEvent::Change);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
