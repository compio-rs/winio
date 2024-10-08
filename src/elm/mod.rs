use std::future::Future;

use futures_channel::mpsc;
use futures_util::{FutureExt, StreamExt};

#[allow(async_fn_in_trait)]
pub trait Component: Sized {
    type Init;
    type Root;
    type Message;
    type Event;

    fn init(counter: Self::Init, root: &Self::Root, sender: &ComponentSender<Self>) -> Self;

    async fn start(&mut self, sender: &ComponentSender<Self>);

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool;

    fn render(&mut self, sender: &ComponentSender<Self>);
}

#[derive(Debug)]
pub struct ComponentSender<T: Component> {
    message_tx: mpsc::UnboundedSender<T::Message>,
    event_tx: mpsc::UnboundedSender<T::Event>,
}

impl<T: Component> Clone for ComponentSender<T> {
    fn clone(&self) -> Self {
        Self {
            message_tx: self.message_tx.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[derive(Debug)]
struct ComponentReceiver<T: Component> {
    message_rx: mpsc::UnboundedReceiver<T::Message>,
    event_rx: mpsc::UnboundedReceiver<T::Event>,
}

fn component_channel<T: Component>() -> (ComponentSender<T>, ComponentReceiver<T>) {
    let (message_tx, message_rx) = mpsc::unbounded();
    let (event_tx, event_rx) = mpsc::unbounded();
    (
        ComponentSender {
            message_tx,
            event_tx,
        },
        ComponentReceiver {
            message_rx,
            event_rx,
        },
    )
}

impl<T: Component> ComponentSender<T> {
    pub fn post(&self, message: T::Message) {
        self.message_tx.unbounded_send(message).unwrap();
    }

    pub fn output(&self, event: T::Event) {
        self.event_tx.unbounded_send(event).unwrap();
    }
}

impl<T: Component> ComponentReceiver<T> {
    async fn recv(&mut self) -> T::Message {
        self.message_rx.next().await.unwrap()
    }

    fn try_recv(&mut self) -> Option<T::Message> {
        self.message_rx.try_next().ok().flatten()
    }

    async fn recv_output(&mut self) -> T::Event {
        self.event_rx.next().await.unwrap()
    }

    fn try_recv_output(&mut self) -> Option<T::Event> {
        self.event_rx.try_next().ok().flatten()
    }
}

pub struct App {
    runtime: Runtime,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    pub fn run<T: Component>(&mut self, counter: T::Init, root: &T::Root) -> T::Event {
        self.block_on(async {
            let (sender, mut receiver) = component_channel();
            let mut model = T::init(counter, root, &sender);
            loop {
                let fut_start = model.start(&sender);
                let fut_recv = receiver.recv();
                futures_util::select! {
                    _ = fut_start.fuse() => unreachable!(),
                    msg = fut_recv.fuse() => {
                        let need_render = model.update(msg, &sender).await;
                        if need_render {
                            model.render(&sender);
                        }
                        if let Some(e) = receiver.try_recv_output() {
                            break e;
                        }
                    }
                }
            }
        })
    }
}

mod child;
pub use child::*;

mod window;
pub use window::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod canvas;
pub use canvas::*;

use crate::runtime::Runtime;