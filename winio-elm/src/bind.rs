use std::fmt::Debug;

use slab::Slab;

use crate::{Component, ComponentSender};

/// A property that will notify its listeners when its value changes.
pub struct Prop<T> {
    listeners: Slab<Box<dyn Fn(T)>>,
    value: T,
}

impl<T: Debug> Debug for Prop<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Prop")
            .field("value", &self.value)
            .finish_non_exhaustive()
    }
}

impl<T> Prop<T> {
    /// Create a new property with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            listeners: Slab::new(),
            value,
        }
    }

    /// Get the current value of the property.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Unbind a listener by its ID.
    pub fn unbind(&mut self, id: usize) {
        let _ = self.listeners.remove(id);
    }
}

impl<T: Clone + 'static> Prop<T> {
    /// Bind to a component sender, so that when the property is notified, a
    /// message is sent to the component.
    ///
    /// Returns the ID of the listener, which can be used to unbind later.
    pub fn bind<C: Component + 'static>(
        &mut self,
        sender: &ComponentSender<C>,
        f: impl Fn(T) -> C::Message + 'static,
    ) -> usize {
        sender.post(f(self.value.clone()));
        let sender = sender.clone();
        self.listeners.insert(Box::new(move |value| {
            let msg = f(value);
            sender.post(msg);
        }))
    }
}

impl<T: Clone + PartialEq> Prop<T> {
    /// Set the value of the property.
    pub fn set(&mut self, value: T) {
        if value != self.value {
            self.value = value;
            for (_, listener) in &self.listeners {
                listener(self.value.clone());
            }
        }
    }
}
