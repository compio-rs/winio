use crate::{
    BorrowedWindow, Component, ComponentSender, Enable, Layoutable, Point, Size, Visible, ui,
};

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

impl Enable for RadioButton {
    fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
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
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::RadioButton::new(init);
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
pub struct RadioButtonGroup<T> {
    radios: T,
}

impl<'a, T: AsMut<[&'a mut RadioButton]>> RadioButtonGroup<T> {
    /// Create [`RadioButtonGroup`].
    pub fn new(radios: T) -> Self {
        Self { radios }
    }

    /// Start listening the click events of the radio boxes.
    pub async fn start<C: Component>(
        &mut self,
        sender: &ComponentSender<C>,
        mut f: impl FnMut(usize) -> Option<C::Message>,
        _propagate: impl FnMut() -> C::Message,
    ) {
        loop {
            let ((), index, _) = futures_util::future::select_all(
                self.radios
                    .as_mut()
                    .iter()
                    .map(|r| Box::pin(r.widget.wait_click())),
            )
            .await;
            for (i, r) in self.radios.as_mut().iter_mut().enumerate() {
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
