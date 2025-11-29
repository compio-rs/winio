//! ELM primitives for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "gen_blocks", feature(async_iterator, gen_blocks))]
#![warn(missing_docs)]

use smallvec::SmallVec;

/// Foundamental GUI component.
#[allow(async_fn_in_trait)]
pub trait Component: Sized {
    /// Initial parameter type.
    type Init<'a>;
    /// The input message type to update.
    type Message;
    /// The output event type to the parent.
    type Event;
    /// The error type.
    type Error: std::fmt::Debug;

    /// Create the initial component.
    fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self, Self::Error>;

    /// Start the event listening.
    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let _ = sender;
        std::future::pending().await
    }

    /// Respond to the message. Return true if need render.
    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool, Self::Error> {
        let _ = message;
        let _ = sender;
        Ok(false)
    }

    /// Render the widgets.
    fn render(&mut self, sender: &ComponentSender<Self>) -> Result<(), Self::Error> {
        let _ = sender;
        Ok(())
    }

    /// Update the children components. Return true if any child needs render.
    async fn update_children(&mut self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    /// Render the children components. It will be called if any child or self
    /// needs render.
    fn render_children(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum ComponentMessage<M, E> {
    Message(M),
    Event(E),
}

/// Sender of input messages and output events.
#[derive(Debug)]
#[repr(transparent)]
pub struct ComponentSender<T: Component>(Channel<ComponentMessage<T::Message, T::Event>>);

impl<T: Component> ComponentSender<T> {
    pub(crate) fn new() -> Self {
        Self(Channel::new())
    }

    pub(crate) fn from_ref(c: &Channel<ComponentMessage<T::Message, T::Event>>) -> &Self {
        // Safety: repr(transparent)
        unsafe { std::mem::transmute(c) }
    }

    /// Post the message to the queue.
    pub fn post(&self, message: T::Message) {
        self.0.send(ComponentMessage::Message(message))
    }

    /// Post the event to the queue.
    pub fn output(&self, event: T::Event) {
        self.0.send(ComponentMessage::Event(event))
    }

    pub(crate) async fn wait(&self) {
        self.0.wait().await
    }

    pub(crate) fn fetch_all(&self) -> SmallVec<[ComponentMessage<T::Message, T::Event>; 1]> {
        self.0.fetch_all()
    }

    /// Cast the sender for a different component type with the same message and
    /// event types.
    pub fn cast<U: Component<Message = T::Message, Event = T::Event>>(
        &self,
    ) -> &ComponentSender<U> {
        ComponentSender::from_ref(&self.0)
    }
}

impl<T: Component> Clone for ComponentSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

mod channel;
use channel::*;

mod child;
pub use child::*;

mod collection;
pub use collection::*;

mod macros;
pub use macros::*;

mod run;
pub use run::*;

#[cfg(feature = "gen_blocks")]
mod stream;
