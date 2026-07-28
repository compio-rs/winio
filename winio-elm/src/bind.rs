use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use slab::Slab;

use crate::{Component, ComponentSender};

/// A property source that can notify listeners of changes to its value.
pub struct PropSource<T> {
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
    /// Create [`PropSource`].
    pub fn new() -> Self {
        Self {
            listeners: Slab::new(),
        }
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

    /// Unbind a listener by its ID.
    pub fn unbind(&mut self, id: usize) {
        let _ = self.listeners.remove(id);
    }
}

impl<T: 'static> PropSource<T> {
    /// Bind to a [`PropSink`], so that when the property is notified, a message
    /// [`PropSinkMessage::Set`] is sent to the sink.
    pub fn bind_sink(&mut self, sink: &PropSink<T>) -> usize {
        self.bind(&sink.sender, PropSinkMessage::Set)
    }
}

impl<T: Clone> PropSource<T> {
    /// Notify all listeners of a change to the property value.
    pub fn notify(&mut self, value: &T) {
        for (_, listener) in &self.listeners {
            listener(value.clone());
        }
    }
}

/// A property sink that can receive messages to set its value.
pub struct PropSink<T> {
    sender: ComponentSender<Self>,
    _p: PhantomData<T>,
}

impl<T> Debug for PropSink<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropSink").finish_non_exhaustive()
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
pub enum PropSinkEvent<T> {
    /// The value of the property has changed.
    Changed(T),
}

impl<T> Component for PropSink<T> {
    type Error = std::convert::Infallible;
    type Event = PropSinkEvent<T>;
    type Init<'a> = ();
    type Message = PropSinkMessage<T>;

    async fn init(
        _init: Self::Init<'_>,
        sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: sender.clone(),
            _p: PhantomData,
        })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool, Self::Error> {
        match message {
            PropSinkMessage::Set(value) => {
                sender.output(PropSinkEvent::Changed(value));
                Ok(false)
            }
        }
    }
}

/// A property that can be both a source and a sink.
#[derive(Debug)]
pub struct Prop<T> {
    source: PropSource<T>,
    sink: PropSink<T>,
}

impl<T> Prop<T> {
    /// See [`PropSource::bind`].
    pub fn bind<C: Component + 'static>(
        &mut self,
        sender: &ComponentSender<C>,
        f: impl Fn(T) -> C::Message + 'static,
    ) -> usize {
        self.source.bind(sender, f)
    }

    /// See [`PropSource::unbind`].
    pub fn unbind(&mut self, id: usize) {
        self.source.unbind(id);
    }
}

impl<T: 'static> Prop<T> {
    /// See [`PropSource::bind_sink`].
    pub fn bind_sink(&mut self, sink: &PropSink<T>) -> usize {
        self.source.bind_sink(sink)
    }
}

impl<T> Deref for Prop<T> {
    type Target = PropSink<T>;

    fn deref(&self) -> &Self::Target {
        &self.sink
    }
}

impl<T> DerefMut for Prop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sink
    }
}

impl<T: Clone> Component for Prop<T> {
    type Error = std::convert::Infallible;
    type Event = PropSinkEvent<T>;
    type Init<'a> = ();
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
                self.source.notify(value);
            }
        }
        self.sink.update(message, sender.cast()).await
    }
}
