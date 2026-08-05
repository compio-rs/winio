use std::fmt::Debug;

use slab::Slab;

use crate::{Component, ComponentSender};

struct PropSource<T> {
    listeners: Slab<Box<dyn Fn(T)>>,
}

impl<T> Debug for PropSource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropSource").finish_non_exhaustive()
    }
}

impl Default for PropSource<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PropSource<T> {
    pub fn new() -> Self {
        Self {
            listeners: Slab::new(),
        }
    }

    pub fn bind<C: Component + 'static>(
        &mut self,
        current: T,
        sender: &ComponentSender<C>,
        f: impl Fn(T) -> C::Message + 'static,
    ) -> usize {
        sender.post(f(current));
        let sender = sender.clone();
        self.listeners.insert(Box::new(move |value| {
            let msg = f(value);
            sender.post(msg);
        }))
    }

    pub fn unbind(&mut self, id: usize) {
        let _ = self.listeners.remove(id);
    }
}

impl<T: Clone> PropSource<T> {
    pub fn notify(&mut self, value: &T) {
        for (_, listener) in &self.listeners {
            listener(value.clone());
        }
    }
}

/// A property that can be both a source and a sink.
#[derive(Debug)]
pub struct Prop<T> {
    source: PropSource<T>,
    value: T,
}

impl<T> Prop<T> {
    /// Create a new property with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            source: PropSource::new(),
            value,
        }
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
        self.source.bind(self.value.clone(), sender, f)
    }
}

impl<T: Clone + PartialEq> Prop<T> {
    /// Set the value of the property.
    pub fn set(&mut self, value: T) {
        if value != self.value {
            self.value = value;
            self.source.notify(&self.value);
        }
    }
}
