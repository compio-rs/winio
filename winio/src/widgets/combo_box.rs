use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, ObservableVecEvent};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A simple combo box.
#[derive(Debug)]
pub struct ComboBox {
    widget: sys::ComboBox,
}

#[inherit_methods(from = "self.widget")]
impl ComboBox {
    /// The selection index.
    pub fn selection(&self) -> Option<usize>;

    /// Set or cancel the selection.
    pub fn set_selection(&mut self, i: Option<usize>);

    /// If the combo box is editable.
    pub fn is_editable(&self) -> bool;

    /// Set if the combo box is editable.
    pub fn set_editable(&mut self, v: bool);

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
impl Visible for ComboBox {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for ComboBox {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ComboBox {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point) {
        if !super::approx_eq_point(self.loc(), p) {
            self.widget.set_loc(p);
        }
    }

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size) {
        if !super::approx_eq_size(self.size(), v) {
            self.widget.set_size(v);
        }
    }

    fn preferred_size(&self) -> Size;
}

/// Events of [`ComboBox`].
#[non_exhaustive]
pub enum ComboBoxEvent {
    /// The selection has changed.
    Select,
    /// The text has been changed.
    Change,
}

/// Messages of [`ComboBox`].
#[non_exhaustive]
pub enum ComboBoxMessage {
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

impl ComboBoxMessage {
    /// Retrive [`ComboBoxMessage`] from [`ObservableVecEvent`] by custom
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

impl Component for ComboBox {
    type Event = ComboBoxEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ComboBoxMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::ComboBox::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_select = async {
            loop {
                self.widget.wait_select().await;
                sender.output(ComboBoxEvent::Select);
            }
        };
        let fut_change = async {
            loop {
                self.widget.wait_change().await;
                sender.output(ComboBoxEvent::Change);
            }
        };
        futures_util::future::join(fut_select, fut_change).await.0
    }

    async fn update(&mut self, message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        match message {
            ComboBoxMessage::Insert { at, value } => self.insert(at, value),
            ComboBoxMessage::Remove { at } => self.remove(at),
            ComboBoxMessage::Replace { at, value } => self.set(at, value),
            ComboBoxMessage::Clear => self.clear(),
        }
        true
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(ComboBox, widget);
