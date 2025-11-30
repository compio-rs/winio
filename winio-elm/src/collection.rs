use std::ops::Deref;

use crate::{Component, ComponentSender};

/// An observable vector. It outputs events after being changed.
pub struct ObservableVec<T: Clone> {
    vec: Vec<T>,
    sender: ComponentSender<Self>,
}

impl<T: Clone> ObservableVec<T> {
    /// Appends an element to the back of a collection.
    pub fn push(&mut self, v: T) {
        let at = self.vec.len();
        self.vec.push(v.clone());
        self.sender
            .output(ObservableVecEvent::Insert { at, value: v });
    }

    /// Inserts an element at specified position.
    pub fn insert(&mut self, i: usize, v: T) {
        self.vec.insert(i, v.clone());
        self.sender
            .output(ObservableVecEvent::Insert { at: i, value: v });
    }

    /// Removes the last element.
    pub fn pop(&mut self) -> Option<T> {
        let res = self.vec.pop();
        if let Some(v) = res.clone() {
            self.sender.output(ObservableVecEvent::Remove {
                at: self.vec.len(),
                value: v,
            });
        }
        res
    }

    /// Removes and returns the element at specified position.
    pub fn remove(&mut self, i: usize) -> T {
        let res = self.vec.remove(i);
        self.sender.output(ObservableVecEvent::Remove {
            at: i,
            value: res.clone(),
        });
        res
    }

    /// Replaces the element at specified position, and return the old value.
    pub fn replace(&mut self, i: usize, v: T) -> T {
        let element = &mut self.vec[i];
        let old = std::mem::replace(element, v.clone());
        self.sender.output(ObservableVecEvent::Replace {
            at: i,
            old: old.clone(),
            new: v,
        });
        old
    }

    /// Clears the vector.
    pub fn clear(&mut self) {
        self.vec.clear();
        self.sender.output(ObservableVecEvent::Clear);
    }

    /// Shrinks the capacity of the vector as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit();
    }

    /// Length of the vector.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Checks if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Gets the inner items.
    pub fn items(&self) -> &[T] {
        &self.vec
    }

    /// Clears the vector, and appends the items one by one.
    pub fn set_items<U: Into<T>>(
        &mut self,
        items: impl IntoIterator<Item = U>,
    ) -> Result<(), std::convert::Infallible> {
        self.clear();
        for it in items {
            self.push(it.into());
        }
        Ok(())
    }
}

impl<T: Clone> Deref for ObservableVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

/// The events of [`ObservableVec`].
#[derive(Debug)]
pub enum ObservableVecEvent<T> {
    /// An element inserted.
    Insert {
        /// The insert position.
        at: usize,
        /// The value.
        value: T,
    },
    /// An element removed.
    Remove {
        /// The remove position
        at: usize,
        /// The value.
        value: T,
    },
    /// An element of specific position is replaced.
    Replace {
        /// The replace position.
        at: usize,
        /// The old value.
        old: T,
        /// The new value.
        new: T,
    },
    /// The vector has been cleared.
    Clear,
}

/// The messages of [`ObservableVec`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ObservableVecMessage {}

impl<T: Clone> Component for ObservableVec<T> {
    type Error = std::convert::Infallible;
    type Event = ObservableVecEvent<T>;
    type Init<'a> = Vec<T>;
    type Message = ObservableVecMessage;

    async fn init(
        init: Self::Init<'_>,
        sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error> {
        let mut this = Self {
            vec: vec![],
            sender: sender.clone(),
        };
        this.set_items(init)?;
        Ok(this)
    }
}
