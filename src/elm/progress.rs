use crate::{
    BorrowedWindow, Component, ComponentSender, Enable, Layoutable, Point, Size, Visible, ui,
};

/// A progress bar.
#[derive(Debug)]
pub struct Progress {
    widget: ui::Progress,
}

impl Progress {
    /// Value range.
    pub fn range(&self) -> (usize, usize) {
        self.widget.range()
    }

    /// Set the value range.
    pub fn set_range(&mut self, min: usize, max: usize) {
        self.widget.set_range(min, max);
    }

    /// Current position.
    pub fn pos(&self) -> usize {
        self.widget.pos()
    }

    /// Set current position.
    pub fn set_pos(&mut self, pos: usize) {
        self.widget.set_pos(pos);
    }

    /// Get if the progress bar is in indeterminate state.
    pub fn is_indeterminate(&self) -> bool {
        self.widget.is_indeterminate()
    }

    /// Set if the progress bar is in indeterminate state.
    pub fn set_indeterminate(&mut self, v: bool) {
        self.widget.set_indeterminate(v);
    }
}

impl Visible for Progress {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Enable for Progress {
    fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
    }
}

impl Layoutable for Progress {
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

/// Events of [`Progress`].
#[non_exhaustive]
pub enum ProgressEvent {}

impl Component for Progress {
    type Event = ProgressEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Progress::new(init);
        Self { widget }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) {}

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
