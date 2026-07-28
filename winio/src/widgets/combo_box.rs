use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, ObservableVecEvent, Prop, PropSink, PropSinkEvent,
    PropSinkMessage, start,
};
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
    text_prop: Child<Prop<String>>,
    selection_prop: Child<Prop<Option<usize>>>,
    editable_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
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

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&mut self) -> &mut Prop<String> {
        &mut self.text_prop
    }

    /// Property for [`ComboBox::selection`].
    pub fn selection_prop(&mut self) -> &mut Prop<Option<usize>> {
        &mut self.selection_prop
    }

    /// Property for [`ComboBox::is_editable`].
    pub fn editable_prop(&self) -> &PropSink<bool> {
        &self.editable_prop
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
    /// The selection has been changed by user.
    ChangeInputSelection,
    /// The selection prop has been changed.
    ChangePropSelection,
    /// The text has been changed by user input.
    ChangeInputText,
    /// The text prop has been changed.
    ChangePropText,
    /// The editable state has been changed.
    ChangeEditable,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
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

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ComboBox::new(init)?;
        let Ok(text_prop) = Child::<Prop<String>>::init(String::new()).await;
        let Ok(selection_prop) = Child::<Prop<Option<usize>>>::init(None).await;
        let Ok(editable_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            text_prop,
            selection_prop,
            editable_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_select = async {
            loop {
                self.widget.wait_select().await;
                sender.post(ComboBoxMessage::ChangeInputSelection);
            }
        };
        let fut_change = async {
            loop {
                self.widget.wait_change().await;
                sender.post(ComboBoxMessage::ChangeInputText);
            }
        };
        let fut_props = async {
            start! {
                sender, default: ComboBoxMessage::Noop,
                self.selection_prop => { PropSinkEvent::Changed => ComboBoxMessage::ChangePropSelection },
                self.text_prop => { PropSinkEvent::Changed => ComboBoxMessage::ChangePropText },
                self.editable_prop => { PropSinkEvent::Changed => ComboBoxMessage::ChangeEditable },
                self.enabled_prop => { PropSinkEvent::Changed => ComboBoxMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => ComboBoxMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => ComboBoxMessage::ChangeTooltip },
            }
        };
        futures_util::future::join3(fut_select, fut_change, fut_props)
            .await
            .0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.selection_prop.update().await;
        let Ok(r2) = self.editable_prop.update().await;
        let Ok(r3) = self.enabled_prop.update().await;
        let Ok(r4) = self.visible_prop.update().await;
        let Ok(r5) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ComboBoxMessage::Noop => Ok(false),
            ComboBoxMessage::Insert { at, value } => {
                self.insert(at, value)?;
                Ok(true)
            }
            ComboBoxMessage::Remove { at } => {
                self.remove(at)?;
                Ok(true)
            }
            ComboBoxMessage::Replace { at, value } => {
                self.set(at, value)?;
                Ok(true)
            }
            ComboBoxMessage::Clear => {
                self.clear()?;
                Ok(true)
            }
            ComboBoxMessage::ChangeInputSelection => {
                let selection = self.widget.selection()?;
                self.selection_prop.post(PropSinkMessage::Set(selection));
                sender.output(ComboBoxEvent::Select);
                Ok(false)
            }
            ComboBoxMessage::ChangePropSelection => {
                let current = self.widget.selection()?;
                let prop_val = self.selection_prop.get();
                if &current != prop_val
                    && let Some(i) = prop_val
                {
                    self.widget.set_selection(*i)?;
                }
                Ok(true)
            }
            ComboBoxMessage::ChangeInputText => {
                let text = self.widget.text()?;
                self.text_prop.post(PropSinkMessage::Set(text));
                sender.output(ComboBoxEvent::Change);
                Ok(false)
            }
            ComboBoxMessage::ChangePropText => {
                let text = self.widget.text()?;
                if &text != self.text_prop.get() {
                    self.widget.set_text(self.text_prop.get())?;
                }
                Ok(true)
            }
            ComboBoxMessage::ChangeEditable => {
                self.widget.set_editable(**self.editable_prop)?;
                Ok(true)
            }
            ComboBoxMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            ComboBoxMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ComboBoxMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(ComboBox, widget);
