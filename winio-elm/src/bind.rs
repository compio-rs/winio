use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

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

impl<T: PartialEq + 'static> PropSource<T> {
    pub fn bind_sink(&mut self, current: T, sink: &PropSink<T>) -> usize {
        self.bind(current, &sink.sender, PropSinkMessage::Set)
    }
}

impl<T: Clone> PropSource<T> {
    pub fn notify(&mut self, value: &T) {
        for (_, listener) in &self.listeners {
            listener(value.clone());
        }
    }
}

/// A property sink that can receive messages to set its value.
#[derive(Debug)]
pub struct PropSink<T: PartialEq> {
    sender: ComponentSender<Self>,
    value: T,
}

impl<T: PartialEq> PropSink<T> {
    /// Get the current value of the property.
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Messages of [`PropSink`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PropSinkMessage<T> {
    /// Set the value of the property.
    Set(T),
}

/// Events of [`PropSink`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PropSinkEvent {
    /// The value of the property has changed.
    Changed,
}

impl<T: PartialEq> Deref for PropSink<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: PartialEq> Component for PropSink<T> {
    type Error = std::convert::Infallible;
    type Event = PropSinkEvent;
    type Init<'a> = T;
    type Message = PropSinkMessage<T>;

    async fn init(
        init: Self::Init<'_>,
        sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: sender.clone(),
            value: init,
        })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool, Self::Error> {
        match message {
            PropSinkMessage::Set(value) => {
                if value != self.value {
                    self.value = value;
                    sender.output(PropSinkEvent::Changed);
                }
            }
        }
        Ok(false)
    }
}

/// A property that can be both a source and a sink.
#[derive(Debug)]
pub struct Prop<T: PartialEq> {
    source: PropSource<T>,
    sink: PropSink<T>,
}

impl<T: PartialEq> Prop<T> {
    /// Unbind a listener by its ID.
    pub fn unbind(&mut self, id: usize) {
        self.source.unbind(id);
    }
}

impl<T: Clone + PartialEq + 'static> Prop<T> {
    /// Bind to a component sender, so that when the property is notified, a
    /// message is sent to the component.
    ///
    /// Returns the ID of the listener, which can be used to unbind later.
    pub fn bind<C: Component + 'static>(
        &mut self,
        sender: &ComponentSender<C>,
        f: impl Fn(T) -> C::Message + 'static,
    ) -> usize {
        self.source.bind(self.sink.get().clone(), sender, f)
    }

    /// Bind to a [`PropSink`], so that when the property is notified, a message
    /// [`PropSinkMessage::Set`] is sent to the sink.
    pub fn bind_sink(&mut self, sink: &PropSink<T>) -> usize {
        self.source.bind_sink(self.sink.get().clone(), sink)
    }
}

impl<T: PartialEq> Deref for Prop<T> {
    type Target = PropSink<T>;

    fn deref(&self) -> &Self::Target {
        &self.sink
    }
}

impl<T: PartialEq> DerefMut for Prop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sink
    }
}

impl<T: Clone + PartialEq> Component for Prop<T> {
    type Error = std::convert::Infallible;
    type Event = PropSinkEvent;
    type Init<'a> = T;
    type Message = PropSinkMessage<T>;

    async fn init(
        init: Self::Init<'_>,
        sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            source: PropSource::new(),
            sink: PropSink::init(init, sender.cast()).await?,
        })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool, Self::Error> {
        match &message {
            PropSinkMessage::Set(value) => {
                if value != &self.sink.value {
                    self.source.notify(value);
                }
            }
        }
        self.sink.update(message, sender.cast()).await
    }
}
