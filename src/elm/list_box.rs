use super::ObservableVecEvent;
use crate::{
    BorrowedWindow, Component, ComponentSender, Enable, Layoutable, Point, Size, Visible, ui,
};

/// A simple list box.
#[derive(Debug)]
pub struct ListBox {
    widget: ui::ListBox,
}

impl ListBox {
    /// Get the selected state by index.
    pub fn is_selected(&self, i: usize) -> bool {
        self.widget.is_selected(i)
    }

    /// Set the selected state by index.
    pub fn set_selected(&mut self, i: usize, v: bool) {
        self.widget.set_selected(i, v);
    }

    /// The length of selection list.
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// If the selection list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the selection list.
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Get the selection item by index.
    pub fn get(&self, i: usize) -> String {
        self.widget.get(i)
    }

    /// Set the selection item by index.
    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        self.widget.set(i, s)
    }

    /// Insert the selection item by index.
    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        self.widget.insert(i, s);
    }

    /// Remove the selection item by index.
    pub fn remove(&mut self, i: usize) {
        self.widget.remove(i);
    }
}

impl Visible for ListBox {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Enable for ListBox {
    fn is_enabled(&self) -> bool {
        self.widget.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) {
        self.widget.set_enabled(v);
    }
}

impl Layoutable for ListBox {
    fn loc(&self) -> Point {
        self.widget.loc()
    }

    fn set_loc(&mut self, p: Point) {
        if !super::approx_eq_point(self.loc(), p) {
            self.widget.set_loc(p);
        }
    }

    fn size(&self) -> Size {
        self.widget.size()
    }

    fn set_size(&mut self, v: Size) {
        if !super::approx_eq_size(self.size(), v) {
            self.widget.set_size(v);
        }
    }

    fn preferred_size(&self) -> Size {
        self.widget.preferred_size()
    }

    fn min_size(&self) -> Size {
        self.widget.min_size()
    }
}

/// Events of [`ListBox`].
#[non_exhaustive]
pub enum ListBoxEvent {
    /// The selection has changed.
    Select,
}

/// Messages of [`ListBox`].
#[non_exhaustive]
pub enum ListBoxMessage {
    /// An element inserted.
    Insert {
        /// The insert position.
        at: usize,
        /// The value.
        value: String,
    },
    /// An element removed.
    Remove {
        /// The remove position
        at: usize,
    },
    /// An element of specific position is replaced.
    Replace {
        /// The replace position.
        at: usize,
        /// The new value.
        value: String,
    },
    /// The vector has been cleared.
    Clear,
}

impl ListBoxMessage {
    /// Retrive [`ListBoxMessage`] from [`ObservableVecEvent`] by custom
    /// function.
    pub fn from_observable_vec_event_by<T>(
        e: ObservableVecEvent<T>,
        mut f: impl FnMut(T) -> String,
    ) -> Self {
        match e {
            ObservableVecEvent::Insert { at, value } => Self::Insert {
                at,
                value: f(value),
            },
            ObservableVecEvent::Remove { at, .. } => Self::Remove { at },
            ObservableVecEvent::Replace { at, new, .. } => Self::Replace { at, value: f(new) },
            ObservableVecEvent::Clear => Self::Clear,
        }
    }

    /// Retrive [`ComboBoxMessage`] from [`ObservableVecEvent`].
    pub fn from_observable_vec_event<T: ToString>(e: ObservableVecEvent<T>) -> Self {
        Self::from_observable_vec_event_by(e, |v| v.to_string())
    }
}

impl Component for ListBox {
    type Event = ListBoxEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ListBoxMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::ListBox::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        loop {
            self.widget.wait_select().await;
            sender.output(ListBoxEvent::Select);
        }
    }

    async fn update(&mut self, message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        match message {
            ListBoxMessage::Insert { at, value } => self.insert(at, value),
            ListBoxMessage::Remove { at } => self.remove(at),
            ListBoxMessage::Replace { at, value } => self.set(at, value),
            ListBoxMessage::Clear => self.clear(),
        }
        true
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
