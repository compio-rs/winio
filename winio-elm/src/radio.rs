use std::ops::{Deref, DerefMut};

use crate::{Child, Component, ComponentSender, Prop, PropSource};

/// Trait for components with a checked state that can be selected among the
/// siblings of a [`RadioGroup`], like radio buttons or radio menu items.
pub trait Radio: Component {
    /// If the item is checked.
    fn is_checked(&self) -> Result<bool, Self::Error>;

    /// Set the checked state.
    fn set_checked(&mut self, v: bool) -> Result<(), Self::Error>;
}

/// Trait for events that can be treated as clicks by a [`RadioGroup`].
pub trait RadioEvent {
    /// If the event means the item has been clicked.
    fn is_click(&self) -> bool;
}

/// A group of [`Radio`] items that keeps exactly one of them selected.
pub struct RadioGroup<T: Radio> {
    radios: Vec<Child<T>>,
    selection: Option<usize>,
    selection_prop: PropSource<Option<usize>>,
}

/// Events of [`RadioGroup`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioGroupEvent {}

/// Messages of [`RadioGroup`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioGroupMessage {
    /// A radio item has been clicked, with its index.
    Click(usize),
    /// Set the selection.
    SetSelection(Option<usize>),
}

impl<T: Radio> RadioGroup<T> {
    /// Get the index of the selected radio item, or `None` if none is
    /// selected.
    pub fn selection(&self) -> Result<Option<usize>, T::Error> {
        Ok(self.selection)
    }

    /// Set the index of the selected radio item.
    pub fn set_selection(&mut self, i: usize) -> Result<(), T::Error> {
        if Some(i) != self.selection {
            self.selection = Some(i);
            self.selection_prop.notify(Some(i));
            for (idx, r) in self.radios.iter_mut().enumerate() {
                r.set_checked(idx == i)?;
            }
        }
        Ok(())
    }

    /// Property for the selected radio item index.
    pub fn selection_prop(&mut self) -> Result<Prop<'_, Option<usize>>, T::Error> {
        Ok(self.selection_prop.as_prop(self.selection))
    }

    /// Appends a radio item to the back of the group.
    pub fn push(&mut self, v: Child<T>) {
        self.radios.push(v);
        self.fix_selection();
    }

    /// Inserts a radio item at specified position.
    pub fn insert(&mut self, i: usize, v: Child<T>) {
        self.radios.insert(i, v);
        self.fix_selection();
    }

    /// Removes the last radio item.
    pub fn pop(&mut self) -> Option<Child<T>> {
        let res = self.radios.pop();
        self.fix_selection();
        res
    }

    /// Removes and returns the radio item at specified position.
    pub fn remove(&mut self, i: usize) -> Child<T> {
        let res = self.radios.remove(i);
        self.fix_selection();
        res
    }

    /// Clears the group.
    pub fn clear(&mut self) {
        self.radios.clear();
        self.fix_selection();
    }

    /// Length of the group.
    pub fn len(&self) -> usize {
        self.radios.len()
    }

    /// Checks if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.radios.is_empty()
    }

    /// Gets the radio item at specified position.
    pub fn get(&self, i: usize) -> Option<&Child<T>> {
        self.radios.get(i)
    }

    /// Resets the selection if the index is out of bounds.
    fn fix_selection(&mut self) {
        let len = self.radios.len();
        if let Some(i) = self.selection
            && i >= len
        {
            self.selection = None;
            self.selection_prop.notify(None);
        }
    }
}

impl<T> Component for RadioGroup<T>
where
    T: Radio,
    T::Event: RadioEvent,
{
    type Error = T::Error;
    type Event = RadioGroupEvent;
    type Init<'a> = Vec<Child<T>>;
    type Message = RadioGroupMessage;

    async fn init(
        radios: Self::Init<'_>,
        _sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error> {
        let selection_prop = PropSource::new();
        Ok(Self {
            radios,
            selection: None,
            selection_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let futures = self
            .radios
            .iter_mut()
            .enumerate()
            .map(|(i, c)| {
                c.start(sender, move |e| {
                    e.is_click().then_some(RadioGroupMessage::Click(i))
                })
            })
            .collect::<Vec<_>>();
        futures_util::future::join_all(futures).await;
        std::future::pending().await
    }

    async fn update_children(&mut self) -> Result<bool, Self::Error> {
        let r0 = futures_util::future::try_join_all(self.radios.iter_mut().map(|c| c.update()))
            .await
            .map(|v| v.into_iter().any(|b| b))?;
        Ok(r0)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool, Self::Error> {
        match message {
            RadioGroupMessage::Click(i) => {
                if Some(i) != self.selection {
                    self.selection = Some(i);
                    self.selection_prop.notify(Some(i));
                    for (idx, r) in self.radios.iter_mut().enumerate() {
                        r.set_checked(idx == i)?;
                    }
                } else if let Some(r) = self.radios.get_mut(i) {
                    r.set_checked(true)?;
                }
                Ok(false)
            }
            RadioGroupMessage::SetSelection(selection) => {
                if selection != self.selection {
                    self.selection = selection;
                    self.selection_prop.notify(selection);
                    if let Some(i) = selection {
                        for (idx, r) in self.radios.iter_mut().enumerate() {
                            r.set_checked(idx == i)?;
                        }
                    }
                }
                Ok(false)
            }
        }
    }
}

impl<T: Radio> Deref for RadioGroup<T> {
    type Target = [Child<T>];

    fn deref(&self) -> &Self::Target {
        &self.radios
    }
}

impl<T: Radio> DerefMut for RadioGroup<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.radios
    }
}
