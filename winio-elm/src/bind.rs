use std::fmt::Debug;

use slab::Slab;

use crate::{Component, ComponentSender};

/// A property that will notify its listeners on demand.
pub struct PropSource<T> {
    listeners: Slab<Box<dyn Fn(T)>>,
}

impl<T: Debug> Debug for PropSource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropSource").finish_non_exhaustive()
    }
}

impl<T> Default for PropSource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PropSource<T> {
    /// Create a property listener source.
    pub fn new() -> Self {
        Self {
            listeners: Slab::new(),
        }
    }

    /// Create a [`Prop`] with current value.
    pub fn as_prop<'a>(&'a mut self, value: T) -> Prop<'a, T> {
        Prop::new(self, value)
    }

    /// Unbind a listener by its ID.
    pub fn unbind(&mut self, id: usize) {
        let _ = self.listeners.remove(id);
    }

    /// Bind to a component sender, so that when the property is notified, a
    /// message is sent to the component.
    ///
    /// Returns the ID of the listener, which can be used to unbind later.
    pub fn bind<C: Component + 'static>(
        &mut self,
        sender: &ComponentSender<C>,
        f: impl Fn(T) -> C::Message + 'static,
    ) -> usize {
        let sender = sender.clone();
        self.listeners.insert(Box::new(move |value| {
            let msg = f(value);
            sender.post(msg);
        }))
    }
}

impl<T: Clone> PropSource<T> {
    /// Notify all listeners of a new value.
    pub fn notify(&self, value: T) {
        for (_, listener) in &self.listeners {
            listener(value.clone());
        }
    }
}

/// A property that will notify its listeners when its value changes.
pub struct Prop<'a, T> {
    source: &'a mut PropSource<T>,
    value: T,
}

impl<'a, T> Prop<'a, T> {
    fn new(source: &'a mut PropSource<T>, value: T) -> Self {
        Self { source, value }
    }

    /// Get the current value of the property.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Unbind a listener by its ID.
    pub fn unbind(&mut self, id: usize) {
        self.source.unbind(id);
    }
}

impl<'a, T: Clone + 'static> Prop<'a, T> {
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
        self.source.bind(sender, f)
    }
}
