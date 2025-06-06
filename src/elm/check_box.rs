use crate::{Component, ComponentSender, Layoutable, Point, Size, Visible, Window, ui};

/// A simple check box.
#[derive(Debug)]
pub struct CheckBox {
    widget: ui::CheckBox,
}

impl CheckBox {
    /// The text.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the text.
    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }

    /// If the box is checked.
    pub fn is_checked(&self) -> bool {
        self.widget.is_checked()
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool) {
        self.widget.set_checked(v);
    }
}

impl Visible for CheckBox {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Layoutable for CheckBox {
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

/// Events of [`CheckBox`].
#[non_exhaustive]
pub enum CheckBoxEvent {
    /// The check box has been clicked.
    Click,
}

impl Component for CheckBox {
    type Event = CheckBoxEvent;
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_counter: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::CheckBox::new(root);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
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
