use std::hint::unreachable_unchecked;

use futures_util::FutureExt;

/// Foundamental GUI component.
#[allow(async_fn_in_trait)]
pub trait Component: Sized {
    /// Initial parameter type.
    type Init<'a>;
    /// The input message type to update.
    type Message;
    /// The output event type to the parent.
    type Event;

    /// Create the initial component.
    fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self;

    /// Start the event listening.
    async fn start(&mut self, sender: &ComponentSender<Self>) -> !;

    /// Respond to the message.
    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool;

    /// Render the widgets.
    fn render(&mut self, sender: &ComponentSender<Self>);
}

#[derive(Debug)]
pub(crate) enum ComponentMessage<T: Component> {
    Message(T::Message),
    Event(T::Event),
}

/// Sender of input messages and output events.
#[derive(Debug)]
pub struct ComponentSender<T: Component>(Channel<ComponentMessage<T>>);

impl<T: Component> ComponentSender<T> {
    pub(crate) fn new() -> Self {
        Self(Channel::new())
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

    pub(crate) fn fetch_all(&self) -> impl IntoIterator<Item = ComponentMessage<T>> {
        self.0.fetch_all()
    }
}

impl<T: Component> Clone for ComponentSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub async fn run<'a, T: Component>(init: impl Into<T::Init<'a>>) -> T::Event {
    let sender = ComponentSender::new();
    let mut model = T::init(init.into(), &sender);
    model.render(&sender);
    'outer: loop {
        let fut_start = model.start(&sender);
        let fut_recv = sender.wait();
        futures_util::select! {
            // SAFETY: never type
            _ = fut_start.fuse() => unsafe { unreachable_unchecked() },
            _ = fut_recv.fuse() => {
                let mut need_render = false;
                for msg in sender.fetch_all() {
                    need_render |= match msg {
                        ComponentMessage::Message(msg) => model.update(msg, &sender).await,
                        ComponentMessage::Event(e) => break 'outer e,
                    };
                }
                if need_render {
                    model.render(&sender);
                }
            }
        }
    }
}

mod channel;
use channel::*;

mod child;
pub use child::*;

mod macros;
pub use macros::*;
