use async_channel as mpmc;
use futures_util::FutureExt;

#[allow(async_fn_in_trait)]
pub trait Component: Sized {
    type Init;
    type Root;
    type Message;
    type Event;

    fn init(counter: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> Self;

    async fn start(&mut self, sender: ComponentSender<Self>);

    async fn update(&mut self, message: Self::Message, sender: ComponentSender<Self>) -> bool;

    fn render(&mut self, sender: ComponentSender<Self>);
}

#[derive(Debug)]
pub struct ComponentSender<T: Component> {
    message_tx: mpmc::Sender<T::Message>,
    message_rx: mpmc::Receiver<T::Message>,
    event_tx: mpmc::Sender<T::Event>,
    event_rx: mpmc::Receiver<T::Event>,
}

impl<T: Component> Clone for ComponentSender<T> {
    fn clone(&self) -> Self {
        Self {
            message_tx: self.message_tx.clone(),
            message_rx: self.message_rx.clone(),
            event_tx: self.event_tx.clone(),
            event_rx: self.event_rx.clone(),
        }
    }
}

impl<T: Component> ComponentSender<T> {
    fn new() -> Self {
        let (message_tx, message_rx) = mpmc::unbounded();
        let (event_tx, event_rx) = mpmc::unbounded();
        Self {
            message_tx,
            message_rx,
            event_tx,
            event_rx,
        }
    }

    pub async fn post(&self, message: T::Message) {
        self.message_tx.send(message).await.unwrap();
    }

    async fn recv(&self) -> T::Message {
        self.message_rx.recv().await.unwrap()
    }

    pub async fn output(&self, event: T::Event) {
        self.event_tx.send(event).await.unwrap();
    }

    async fn recv_output(&self) -> T::Event {
        self.event_rx.recv().await.unwrap()
    }

    fn try_recv_output(&self) -> Option<T::Event> {
        self.event_rx.try_recv().ok()
    }
}

#[derive(Debug)]
pub struct App {
    #[allow(dead_code)]
    name: String,
}

impl App {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub async fn run<T: Component>(&mut self, counter: T::Init, root: &T::Root) -> T::Event {
        let sender = ComponentSender::new();
        let mut model = T::init(counter, root, sender.clone());
        loop {
            let fut_start = model.start(sender.clone());
            let fut_recv = sender.recv();
            futures_util::select! {
                _ = fut_start.fuse() => unreachable!(),
                msg = fut_recv.fuse() => {
                    let need_render = model.update(msg, sender.clone()).await;
                    if need_render {
                        model.render(sender.clone());
                    }
                    if let Some(e) = sender.try_recv_output() {
                        break e;
                    }
                }
            }
        }
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
