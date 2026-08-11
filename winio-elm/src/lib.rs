//! ELM primitives for winio.
//!
//! This crate provides the core architecture of winio, inspired by the
//! [Elm Architecture](https://guide.elm-lang.org/architecture/): the state of
//! the user interface is kept in a **component**, which reacts to **messages**
//! and produces **events**.
//!
//! # Concepts
//!
//! A [`Component`] is the basic unit of the UI. It can be as simple as a
//! button, or as complex as the whole application. A component:
//!
//! * is created by [`Component::init`] with the initial parameters;
//! * listens to the native input events (mouse clicks, text input, timers, ...)
//!   in [`Component::start`];
//! * reacts to the messages sent to it in [`Component::update`];
//! * draws its widgets in [`Component::render`].
//!
//! Components form a tree: a component may contain several child components,
//! wrapped in [`Child`]. Messages flow from the root down to the leaves,
//! while events flow from the leaves up to the root:
//!
//! # A complete example
//!
//! The following example defines a `Counter` component with a value, which
//! can be incremented or decremented. It shows how to implement
//! [`Component`] for a simple stateful component:
//!
//! ```
//! use winio_elm::{Component, ComponentSender};
//!
//! /// The input messages, sent by the parent component.
//! enum CounterMessage {
//!     Increment,
//!     Decrement,
//! }
//!
//! /// The output events, observed by the parent component.
//! enum CounterEvent {
//!     Changed(i32),
//! }
//!
//! /// The component itself: it just holds a value.
//! struct Counter {
//!     value: i32,
//! }
//!
//! impl Component for Counter {
//!     type Error = std::convert::Infallible;
//!     type Event = CounterEvent;
//!     /// The initial value of the counter.
//!     type Init<'a> = i32;
//!     type Message = CounterMessage;
//!
//!     /// Create the counter with the given initial value.
//!     async fn init(
//!         init: Self::Init<'_>,
//!         _sender: &ComponentSender<Self>,
//!     ) -> Result<Self, Self::Error> {
//!         Ok(Self { value: init })
//!     }
//!
//!     /// React to the messages: modify the state, and notify the parent with
//!     /// an event. The returned `bool` tells the runtime whether the
//!     /// component needs to be re-rendered.
//!     async fn update(
//!         &mut self,
//!         message: Self::Message,
//!         sender: &ComponentSender<Self>,
//!     ) -> Result<bool, Self::Error> {
//!         match message {
//!             CounterMessage::Increment => self.value += 1,
//!             CounterMessage::Decrement => self.value -= 1,
//!         }
//!         sender.output(CounterEvent::Changed(self.value));
//!         Ok(true)
//!     }
//! }
//! ```
//!
//! See [`Child`] for how to compose components, and [`Root`] for how to run
//! the component tree.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "gen_blocks", feature(async_iterator, gen_blocks))]
#![warn(missing_docs)]

use smallvec::SmallVec;

/// Foundamental GUI component.
///
/// A component owns a piece of the UI state, and reacts to the messages sent
/// to it. It is the basic unit of the winio user interface, and can be
/// composed into a tree with [`Child`].
///
/// To implement a component, you need to:
///
/// 1. Define the message and event types;
/// 2. Implement [`Component::init`] to create the component with its initial
///    state;
/// 3. Optionally implement [`Component::start`] to listen to the native input
///    events;
/// 4. Optionally implement [`Component::update`] to react to the messages;
/// 5. Optionally implement [`Component::render`] to draw the widgets.
///
/// See the [crate-level documentation](crate) for a complete example.
///
/// # The associated types
///
/// * [`Component::Init`]: the initial parameters passed to [`Component::init`],
///   when the component is created by [`Child::init`] or [`Root::init`].
/// * [`Component::Message`]: the input messages. They are sent by the parent
///   component (or the user), and handled in [`Component::update`]. Messages
///   should be a small enum describing all the possible commands.
/// * [`Component::Event`]: the output events. They are emitted by the component
///   (usually in [`Component::update`] or [`Component::start`]), and observed
///   by the parent component. Events describe what happened to the component,
///   so that the parent can react accordingly.
/// * [`Component::Error`]: the error type of the fallible operations.
///
/// # The lifecycle methods
///
/// The runtime drives a component in a loop:
///
/// ```text
///  init ──► start ──► (wait for message) ──► update ──► render
///              ▲                                             │
///              └────────────── loop ◄────────────────────────┘
/// ```
///
/// * [`Component::init`]: creates the component. It is called once, before
///   anything else. You should create the native widgets here, and initialize
///   the state.
/// * [`Component::start`]: starts listening to the native input events. It
///   usually runs an infinite loop: wait for an input, then either
///   [`ComponentSender::post`] a message to itself or
///   [`ComponentSender::output`] an event to the parent.
/// * [`Component::update`]: handles one message. It should modify the state,
///   and return `true` if the component needs to be re-rendered.
/// * [`Component::render`]: draws the component itself. It is called when
///   [`Component::update`] returns `true`, or when the component was just
///   created. Note that it is *not* called when only a child needs rendering;
///   but once it is called, [`Component::render_children`] is guaranteed to be
///   called afterwards.
/// * [`Component::update_children`]: updates all the child components. It is
///   called before the messages are handled, and returns `true` if any child
///   needs rendering.
/// * [`Component::render_children`]: renders all the child components. It is
///   called when the component itself needs rendering (right after
///   [`Component::render`]), or when any child needs rendering.
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
    ///
    /// This is the constructor of the component. It is called once by
    /// [`Child::init`] or [`Root::init`] with the initial parameters, and
    /// should return the component with its initial state.
    ///
    /// For a widget component, this is where you create the native widgets
    /// and initialize the properties. The `sender` can be used to bind
    /// properties to the messages, or to spawn with your background tasks.
    async fn init(
        init: Self::Init<'_>,
        sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error>;

    /// Start the event listening.
    ///
    /// This method is called after [`Component::init`], and usually runs an
    /// infinite loop that waits for the native input events. For example, a
    /// button component would wait for the clicks:
    ///
    /// ```ignore
    /// async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
    ///     loop {
    ///         self.widget.wait_click().await;
    ///         sender.post(ButtonMessage::ChangeInputClicked);
    ///     }
    /// }
    /// ```
    ///
    /// The default implementation waits forever, which is suitable for
    /// components that do not listen to any input.
    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let _ = sender;
        std::future::pending().await
    }

    /// Respond to the message. Return true if need render.
    ///
    /// This is the heart of the component. It is called with each message
    /// sent to the component, and should:
    ///
    /// * modify the internal state;
    /// * apply the changes to the native widgets;
    /// * emit the events with [`ComponentSender::output`], if the parent needs
    ///   to know what happened;
    /// * return `true` if the component needs to be re-rendered.
    ///
    /// ```ignore
    /// async fn update(
    ///     &mut self,
    ///     message: Self::Message,
    ///     sender: &ComponentSender<Self>,
    /// ) -> Result<bool, Self::Error> {
    ///     match message {
    ///         ButtonMessage::ChangeInputClicked => {
    ///             sender.output(ButtonEvent::Click);
    ///             Ok(false)
    ///         }
    ///         ButtonMessage::SetText(text) => {
    ///             self.widget.set_text(text)?;
    ///             Ok(true)
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// The default implementation does nothing and returns `false`.
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
    ///
    /// This method draws the component itself, e.g. by calculating the
    /// layout of the child widgets. It is called when:
    ///
    /// * the component was just created;
    /// * [`Component::update`] returns `true`.
    ///
    /// Note that it is *not* called when only a child component needs
    /// rendering. Conversely, once it is called,
    /// [`Component::render_children`] is guaranteed to be called afterwards.
    ///
    /// The default implementation does nothing.
    fn render(&mut self, sender: &ComponentSender<Self>) -> Result<(), Self::Error> {
        let _ = sender;
        Ok(())
    }

    /// Update the children components. Return true if any child needs render.
    ///
    /// This method is called before the messages are handled, and should
    /// update all the child components. Use the
    /// [`update_children!`](crate::update_children) macro to update all the
    /// children in one go:
    ///
    /// ```ignore
    /// async fn update_children(&mut self) -> Result<bool, Self::Error> {
    ///     update_children!(
    ///         self.window,
    ///         self.button,
    ///         self.label,
    ///     )
    /// }
    /// ```
    ///
    /// The default implementation does nothing and returns `false`.
    async fn update_children(&mut self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    /// Render the children components.
    ///
    /// This method should render all the child components, usually by
    /// calculating their layout. It is called when:
    ///
    /// * the component itself needs rendering, right after
    ///   [`Component::render`];
    /// * any child component needs rendering.
    ///
    /// The default implementation does nothing.
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

    pub(crate) fn wake(&self) {
        self.0.wake()
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

mod boxed;
pub use boxed::*;

mod bind;
pub use bind::*;

#[cfg(feature = "gen_blocks")]
mod stream;
