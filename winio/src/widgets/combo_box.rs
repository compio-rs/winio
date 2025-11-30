use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, ObservableVecEvent};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple combo box.
#[derive(Debug)]
pub struct ComboBox {
    widget: sys::ComboBox,
}

impl Failable for ComboBox {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for ComboBox {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for ComboBox {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl ComboBox {
    /// The selection index.
    pub fn selection(&self) -> Result<Option<usize>>;

    /// Set the selection.
    pub fn set_selection(&mut self, i: usize) -> Result<()>;

    /// If the combo box is editable.
    pub fn is_editable(&self) -> Result<bool>;

    /// Set if the combo box is editable.
    pub fn set_editable(&mut self, v: bool) -> Result<()>;

    /// The length of list.
    pub fn len(&self) -> Result<usize>;

    /// If the list is empty.
    pub fn is_empty(&self) -> Result<bool>;

    /// Clear the list.
    pub fn clear(&mut self) -> Result<()>;

    /// Get the item by index.
    pub fn get(&self, i: usize) -> Result<String>;

    /// Set the item by index.
    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()>;

    /// Insert an item by index.
    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()>;

    /// Remove the item by index.
    pub fn remove(&mut self, i: usize) -> Result<()>;

    /// Push an item to the end of the list.
    pub fn push(&mut self, s: impl AsRef<str>) -> Result<()> {
        let len = self.len()?;
        self.insert(len, s)
    }

    /// Clears all items, and appends the new items one by one.
    pub fn set_items<U: Into<String>>(&mut self, items: impl IntoIterator<Item = U>) -> Result<()> {
        self.clear()?;
        for it in items {
            self.push(it.into())?;
        }
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for ComboBox {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for ComboBox {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ComboBox {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()> {
        if !super::approx_eq_point(self.loc()?, p) {
            self.widget.set_loc(p)?;
        }
        Ok(())
    }

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()> {
        if !super::approx_eq_size(self.size()?, v) {
            self.widget.set_size(v)?;
        }
        Ok(())
    }

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`ComboBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ComboBoxEvent {
    /// The selection has changed.
    Select,
    /// The text has been changed.
    Change,
}

/// Messages of [`ComboBox`].
#[derive(Debug)]
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
    type Error = Error;
    type Event = ComboBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ComboBoxMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ComboBox::new(init)?;
        Ok(Self { widget })
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

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ComboBoxMessage::Insert { at, value } => self.insert(at, value)?,
            ComboBoxMessage::Remove { at } => self.remove(at)?,
            ComboBoxMessage::Replace { at, value } => self.set(at, value)?,
            ComboBoxMessage::Clear => self.clear()?,
        }
        Ok(true)
    }
}

winio_handle::impl_as_widget!(ComboBox, widget);
