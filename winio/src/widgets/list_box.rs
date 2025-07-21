use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, ObservableVecEvent};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A simple list box.
#[derive(Debug)]
pub struct ListBox {
    widget: sys::ListBox,
}

#[inherit_methods(from = "self.widget")]
impl ListBox {
    /// Get the selected state by index.
    pub fn is_selected(&self, i: usize) -> bool;

    /// Set the selected state by index.
    pub fn set_selected(&mut self, i: usize, v: bool);

    /// The length of selection list.
    pub fn len(&self) -> usize;

    /// If the selection list is empty.
    pub fn is_empty(&self) -> bool;

    /// Clear the selection list.
    pub fn clear(&mut self);

    /// Get the selection item by index.
    pub fn get(&self, i: usize) -> String;

    /// Set the selection item by index.
    pub fn set(&mut self, i: usize, s: impl AsRef<str>);

    /// Insert the selection item by index.
    pub fn insert(&mut self, i: usize, s: impl AsRef<str>);

    /// Remove the selection item by index.
    pub fn remove(&mut self, i: usize);
}

#[inherit_methods(from = "self.widget")]
impl Visible for ListBox {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for ListBox {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ListBox {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;

    fn min_size(&self) -> Size;
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

    /// Retrive [`ListBoxMessage`] from [`ObservableVecEvent`].
    pub fn from_observable_vec_event<T: ToString>(e: ObservableVecEvent<T>) -> Self {
        Self::from_observable_vec_event_by(e, |v| v.to_string())
    }
}

impl Component for ListBox {
    type Event = ListBoxEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ListBoxMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::ListBox::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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

winio_handle::impl_as_widget!(ListBox, widget);
