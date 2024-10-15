use crate::{Component, ComponentSender, HAlign, Layoutable, Point, Size, Window, ui};

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

    /// The horizontal alignment.
    pub fn halign(&self) -> HAlign {
        self.widget.halign()
    }

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) {
        self.widget.set_halign(align);
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
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_counter: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Edit::new(root);
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

/// A simple single-line password input box.
#[derive(Debug)]
pub struct PasswordEdit {
    widget: ui::PasswordEdit,
}

impl PasswordEdit {
    /// The text.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the text.
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

impl Layoutable for PasswordEdit {
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

impl Component for PasswordEdit {
    type Event = EditEvent;
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_counter: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::PasswordEdit::new(root);
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
