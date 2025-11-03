//! ELM primitives for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

use std::hint::unreachable_unchecked;

use async_stream::stream;
use futures_util::{FutureExt, Stream, StreamExt};

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

    /// Respond to the message. Return true if need render.
    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool;

    /// Render the widgets.
    fn render(&mut self, sender: &ComponentSender<Self>);

    /// Update the children components. Return true if any child needs render.
    async fn update_children(&mut self) -> bool {
        false
    }

    /// Render the children components. It will be called if any child or self
    /// needs render.
    fn render_children(&mut self) {}
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

/// Initiates and runs a root component, and exits when the component outputs an
/// event.
pub async fn run<'a, T: Component>(init: impl Into<T::Init<'a>>) -> T::Event {
    let stream = run_events::<T>(init);
    let mut stream = std::pin::pin!(stream);
    stream.next().await.expect("component exits without event")
}

/// Initiates and runs a root component, and yields the events.
#[deprecated = "renamed to run_events"]
pub fn run_component<'a, T: Component>(
    init: impl Into<T::Init<'a>>,
) -> impl Stream<Item = T::Event> {
    run_events::<T>(init)
}

/// Initiates and runs a root component, and yields the events.
pub fn run_events<'a, T: Component>(init: impl Into<T::Init<'a>>) -> impl Stream<Item = T::Event> {
    stream! {
        let sender = ComponentSender::new();
        let mut model = T::init(init.into(), &sender);
        model.render(&sender);
        for await event in run_events_impl(&mut model, &sender) {
            yield event;
        }
    }
}

fn run_events_impl<'a, T: Component>(
    model: &'a mut T,
    sender: &'a ComponentSender<T>,
) -> impl Stream<Item = T::Event> + 'a {
    stream! {
        loop {
            let fut_start = model.start(sender);
            let fut_recv = sender.wait();
            futures_util::select! {
                // SAFETY: never type
                _ = fut_start.fuse() => unsafe { unreachable_unchecked() },
                _ = fut_recv.fuse() => {
                    let mut need_render = false;
                    let mut children_need_render = model.update_children().await;
                    for msg in sender.fetch_all() {
                        match msg {
                            ComponentMessage::Message(msg) => {
                                need_render |= model.update(msg, sender).await;
                            }
                            ComponentMessage::Event(e) => yield e,
                        };
                    }
                    children_need_render |= need_render;
                    if need_render {
                        model.render(sender);
                    }
                    if children_need_render {
                        model.render_children();
                    }
                }
            }
        }
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
use smallvec::SmallVec;

#[cfg(test)]
mod test {
    use super::*;

    struct TestComponent;

    #[derive(Debug, PartialEq, Eq)]
    enum TestEvent {
        Event1,
        Event2,
    }

    enum TestMessage {
        Msg1,
        Msg2,
    }

    impl Component for TestComponent {
        type Event = TestEvent;
        type Init<'a> = Vec<TestMessage>;
        type Message = TestMessage;

        fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
            for m in init {
                sender.post(m);
            }
            Self
        }

        async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
            std::future::pending().await
        }

        async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
            match message {
                TestMessage::Msg1 => {
                    sender.output(TestEvent::Event1);
                    false
                }
                TestMessage::Msg2 => {
                    sender.output(TestEvent::Event2);
                    false
                }
            }
        }

        fn render(&mut self, _sender: &ComponentSender<Self>) {}
    }

    #[compio::test]
    async fn test_run() {
        let event = run::<TestComponent>(vec![TestMessage::Msg1]).await;
        assert_eq!(event, TestEvent::Event1);

        let event = run::<TestComponent>(vec![TestMessage::Msg2, TestMessage::Msg1]).await;
        assert_eq!(event, TestEvent::Event2);
    }

    #[compio::test]
    async fn test_run_component() {
        let events = run_events::<TestComponent>(vec![
            TestMessage::Msg1,
            TestMessage::Msg2,
            TestMessage::Msg1,
        ]);
        assert_send_sync(&events);
        let expects = [TestEvent::Event1, TestEvent::Event2, TestEvent::Event1];
        let zip = events.zip(futures_util::stream::iter(expects.into_iter()));
        let mut zip = std::pin::pin!(zip);
        while let Some((e, ex)) = zip.next().await {
            assert_eq!(e, ex);
        }
    }

    fn assert_send_sync<T: Send + Sync>(_: &T) {}
}
