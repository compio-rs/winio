use std::hint::unreachable_unchecked;

use async_stream::stream;
use futures_util::{FutureExt, Stream};

use crate::{Component, ComponentMessage, ComponentSender};

/// Events yielded by the [`run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RunEvent<T, E> {
    /// An event emitted by the component.
    Event(T),
    /// An error occurred during initialization.
    ///
    /// This error is not recoverable, and the component will exit.
    InitErr(E),
    /// An error occurred during update.
    UpdateErr(E),
    /// An error occurred during rendering.
    RenderErr(E),
}

impl<T, E> RunEvent<T, E> {
    /// Flatten the [`RunEvent`] into a [`Result`].
    pub fn flatten(self) -> Result<T, E> {
        match self {
            RunEvent::Event(t) => Ok(t),
            RunEvent::InitErr(e) | RunEvent::UpdateErr(e) | RunEvent::RenderErr(e) => Err(e),
        }
    }
}

/// Initiates and runs a root component, and yields the events.
pub fn run_events<'a, T: Component>(
    init: impl Into<T::Init<'a>>,
) -> impl Stream<Item = RunEvent<T::Event, T::Error>> {
    stream! {
        let sender = ComponentSender::new();
        let mut model = match T::init(init.into(), &sender) {
            Ok(m) => m,
            Err(e) => {
                yield RunEvent::InitErr(e);
                // Init error is not recoverable, exit.
                return;
            }
        };
        if let Err(e) = model.render(&sender) {
            yield RunEvent::RenderErr(e);
        }
        if let Err(e) = model.render_children() {
            yield RunEvent::RenderErr(e);
        }
        for await event in run_events_impl(&mut model, &sender) {
            yield event;
        }
    }
}

pub(crate) fn run_events_impl<'a, T: Component>(
    model: &'a mut T,
    sender: &'a ComponentSender<T>,
) -> impl Stream<Item = RunEvent<T::Event, T::Error>> + 'a {
    stream! {
        loop {
            let fut_start = model.start(sender);
            let fut_recv = sender.wait();
            futures_util::select! {
                // SAFETY: never type
                _ = fut_start.fuse() => unsafe { unreachable_unchecked() },
                _ = fut_recv.fuse() => {}
            }
            let mut need_render = false;
            let mut children_need_render = match model.update_children().await {
                Ok(v) => v,
                Err(e) => {
                    yield RunEvent::UpdateErr(e);
                    false
                }
            };
            for msg in sender.fetch_all() {
                match msg {
                    ComponentMessage::Message(msg) => {
                        need_render |= match model.update(msg, sender).await {
                            Ok(v) => v,
                            Err(e) => {
                                yield RunEvent::UpdateErr(e);
                                false
                            }
                        };
                    }
                    ComponentMessage::Event(e) => yield RunEvent::Event(e),
                };
            }
            children_need_render |= need_render;
            if need_render {
                if let Err(e) = model.render(sender) {
                    yield RunEvent::RenderErr(e);
                }
            }
            if children_need_render {
                if let Err(e) = model.render_children() {
                    yield RunEvent::RenderErr(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use futures_util::StreamExt;

    use crate::*;

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
        type Error = ();
        type Event = TestEvent;
        type Init<'a> = Vec<TestMessage>;
        type Message = TestMessage;

        fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self, ()> {
            for m in init {
                sender.post(m);
            }
            Ok(Self)
        }

        async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
            std::future::pending().await
        }

        async fn update(
            &mut self,
            message: Self::Message,
            sender: &ComponentSender<Self>,
        ) -> Result<bool, ()> {
            match message {
                TestMessage::Msg1 => {
                    sender.output(TestEvent::Event1);
                    Ok(false)
                }
                TestMessage::Msg2 => {
                    sender.output(TestEvent::Event2);
                    Ok(false)
                }
            }
        }

        fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<(), ()> {
            Ok(())
        }
    }

    async fn run_once<'a, T: Component>(
        init: impl Into<T::Init<'a>>,
    ) -> RunEvent<T::Event, T::Error> {
        let stream = run_events::<T>(init);
        let mut stream = std::pin::pin!(stream);
        stream.next().await.expect("component exits without event")
    }

    #[compio::test]
    async fn test_run() {
        let event = run_once::<TestComponent>(vec![TestMessage::Msg1]).await;
        assert_eq!(event, RunEvent::Event(TestEvent::Event1));

        let event = run_once::<TestComponent>(vec![TestMessage::Msg2, TestMessage::Msg1]).await;
        assert_eq!(event, RunEvent::Event(TestEvent::Event2));
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
            assert_eq!(e, RunEvent::Event(ex));
        }
    }

    fn assert_send_sync<T: Send + Sync>(_: &T) {}
}
