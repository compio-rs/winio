use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, ObservableVecEvent, PropSink, PropSinkEvent, start,
};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple list box.
#[derive(Debug)]
pub struct ListBox {
    widget: sys::ListBox,
    multiple_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
}

impl Failable for ListBox {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for ListBox {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl ListBox {
    /// Get if the list box allows multiple selection.
    pub fn is_multiple(&self) -> Result<bool>;

    /// Set if the list box allows multiple selection.
    pub fn set_multiple(&mut self, v: bool) -> Result<()>;

    /// Get the selected state by index.
    pub fn is_selected(&self, i: usize) -> Result<bool>;

    /// Set the selected state by index.
    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()>;

    /// The length of the list.
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

    /// Property for [`ListBox::is_multiple`].
    pub fn multiple_prop(&self) -> &PropSink<bool> {
        &self.multiple_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for ListBox {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for ListBox {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ListBox {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;

    fn min_size(&self) -> Result<Size>;
}

/// Events of [`ListBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ListBoxEvent {
    /// The selection has changed.
    Select,
}

/// Messages of [`ListBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ListBoxMessage {
    /// No operation.
    Noop,
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
    /// The multiple selection state has been changed.
    ChangeMultiple,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
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
    type Error = Error;
    type Event = ListBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ListBoxMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ListBox::new(init)?;
        let Ok(multiple_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            multiple_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_select = async {
            loop {
                self.widget.wait_select().await;
                sender.output(ListBoxEvent::Select);
            }
        };
        let fut_props = async {
            start! {
                sender, default: ListBoxMessage::Noop,
                self.multiple_prop => { PropSinkEvent::Changed => ListBoxMessage::ChangeMultiple },
                self.enabled_prop => { PropSinkEvent::Changed => ListBoxMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => ListBoxMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => ListBoxMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_select, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.multiple_prop.update().await;
        let Ok(r1) = self.enabled_prop.update().await;
        let Ok(r2) = self.visible_prop.update().await;
        let Ok(r3) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ListBoxMessage::Noop => Ok(false),
            ListBoxMessage::Insert { at, value } => {
                self.insert(at, value)?;
                Ok(true)
            }
            ListBoxMessage::Remove { at } => {
                self.remove(at)?;
                Ok(true)
            }
            ListBoxMessage::Replace { at, value } => {
                self.set(at, value)?;
                Ok(true)
            }
            ListBoxMessage::Clear => {
                self.clear()?;
                Ok(true)
            }
            ListBoxMessage::ChangeMultiple => {
                self.widget.set_multiple(**self.multiple_prop)?;
                Ok(true)
            }
            ListBoxMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            ListBoxMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ListBoxMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(ListBox, widget);
