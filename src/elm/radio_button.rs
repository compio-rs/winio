use crate::{Component, ComponentSender, Layoutable, Point, Size, Visible, Window, ui};

/// A simple radio box. See [`RadioButtonGroup`] for making selection groups.
#[derive(Debug)]
pub struct RadioButton {
    widget: ui::RadioButton,
}

impl RadioButton {
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

impl Visible for RadioButton {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Layoutable for RadioButton {
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

/// Events of [`RadioButton`].
#[non_exhaustive]
pub enum RadioButtonEvent {
    /// The check box has been clicked.
    Click,
}

impl Component for RadioButton {
    type Event = RadioButtonEvent;
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_init: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::RadioButton::new(root);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        loop {
            self.widget.wait_click().await;
            sender.output(RadioButtonEvent::Click);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

/// A group of [`RadioButton`]. Only one of them could be checked.
pub struct RadioButtonGroup<'a> {
    radios: Vec<&'a mut RadioButton>,
}

impl<'a> RadioButtonGroup<'a> {
    /// Create [`RadioButtonGroup`].
    pub fn new(radios: Vec<&'a mut RadioButton>) -> Self {
        Self { radios }
    }

    /// Start listening the click events of the radio boxes.
    pub async fn start<C: Component>(
        &mut self,
        sender: &ComponentSender<C>,
        mut f: impl FnMut(usize) -> Option<C::Message>,
    ) {
        loop {
            let ((), index, _) = futures_util::future::select_all(
                self.radios.iter().map(|r| Box::pin(r.widget.wait_click())),
            )
            .await;
            for (i, r) in self.radios.iter_mut().enumerate() {
                if i != index {
                    r.set_checked(false);
                }
            }
            if let Some(message) = f(index) {
                sender.post(message);
            }
        }
    }
}
