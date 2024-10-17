use crate::{Component, ComponentSender, Layoutable, Point, Size, Window, ui};

/// A simple radio box. See [`RadioBoxGroup`] for making selection groups.
#[derive(Debug)]
pub struct RadioBox {
    widget: ui::RadioBox,
}

impl RadioBox {
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

impl Layoutable for RadioBox {
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

/// Events of [`RadioBox`].
#[non_exhaustive]
pub enum RadioBoxEvent {
    /// The check box has been clicked.
    Click,
}

impl Component for RadioBox {
    type Event = RadioBoxEvent;
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_counter: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::RadioBox::new(root);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        loop {
            self.widget.wait_click().await;
            sender.output(RadioBoxEvent::Click);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

/// A group of [`RadioBox`]. Only one of them could be checked.
pub struct RadioBoxGroup<'a> {
    radios: Vec<&'a mut RadioBox>,
}

impl<'a> RadioBoxGroup<'a> {
    /// Create [`RadioBoxGroup`].
    pub fn new(radios: Vec<&'a mut RadioBox>) -> Self {
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
