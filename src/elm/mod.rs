use std::future::Future;

use futures_channel::mpsc;
use futures_util::{FutureExt, StreamExt};

use crate::{Point, Size, runtime::Runtime};

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
    async fn start(&mut self, sender: &ComponentSender<Self>);

    /// Respond to the message.
    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool;

    /// Render the widgets.
    fn render(&mut self, sender: &ComponentSender<Self>);
}

/// Sender of input messages and output events.
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
struct ComponentReceiver<T>(mpsc::UnboundedReceiver<T>);

fn component_channel<T: Component>() -> (
    ComponentSender<T>,
    ComponentReceiver<T::Message>,
    ComponentReceiver<T::Event>,
) {
    let (message_tx, message_rx) = mpsc::unbounded();
    let (event_tx, event_rx) = mpsc::unbounded();
    (
        ComponentSender {
            message_tx,
            event_tx,
        },
        ComponentReceiver(message_rx),
        ComponentReceiver(event_rx),
    )
}

impl<T: Component> ComponentSender<T> {
    /// Post the message to the queue.
    pub fn post(&self, message: T::Message) -> bool {
        self.message_tx.unbounded_send(message).is_ok()
    }

    /// Post the event to the queue.
    pub fn output(&self, event: T::Event) -> bool {
        self.event_tx.unbounded_send(event).is_ok()
    }
}

impl<T> ComponentReceiver<T> {
    async fn recv(&mut self) -> T {
        self.0.next().await.unwrap()
    }

    fn try_recv(&mut self) -> Option<T> {
        self.0.try_next().ok().flatten()
    }
}

/// Root application, manages the async runtime.
pub struct App {
    runtime: Runtime,
    name: Option<String>,
}

impl App {
    /// Create [`App`].
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
            name: None,
        }
    }

    /// Create [`App`] with application name.
    pub fn new_with_name(name: impl AsRef<str>) -> Self {
        #[allow(unused_mut)]
        let mut runtime = Runtime::new();
        let name = name.as_ref().to_string();
        #[cfg(not(any(windows, target_vendor = "apple")))]
        runtime.set_app_id(&name);
        Self {
            runtime,
            name: Some(name),
        }
    }

    /// The application name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Block on the future till it completes.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Create and manage the component, till it posts an event. The application
    /// returns the first event from the component.
    pub fn run<'a, T: Component>(&mut self, init: impl Into<T::Init<'a>>) -> T::Event {
        self.block_on(async {
            let (sender, mut msg_recv, mut ev_recv) = component_channel();
            let mut model = T::init(init.into(), &sender);
            model.render(&sender);
            loop {
                let fut_start = model.start(&sender);
                let fut_recv = msg_recv.recv();
                let fut_ev = ev_recv.recv();
                futures_util::select! {
                    _ = fut_start.fuse() => unreachable!(),
                    msg = fut_recv.fuse() => {
                        let mut need_render = model.update(msg, &sender).await;
                        while let Some(msg) = msg_recv.try_recv() {
                            need_render |= model.update(msg, &sender).await;
                        }
                        if need_render {
                            model.render(&sender);
                        }
                    }
                    e = fut_ev.fuse() => {
                        break e;
                    }
                }
            }
        })
    }
}

fn approx_eq_point(p1: Point, p2: Point) -> bool {
    approx_eq(p1.x, p2.x) && approx_eq(p1.y, p2.y)
}

fn approx_eq_size(s1: Size, s2: Size) -> bool {
    approx_eq(s1.width, s2.width) && approx_eq(s1.height, s2.height)
}

fn approx_eq(f1: f64, f2: f64) -> bool {
    (f1 - f2).abs() < 1.0
}

mod child;
pub use child::*;

mod layout;
pub use layout::*;

mod collection;
pub use collection::*;

mod window;
pub use window::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod canvas;
pub use canvas::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;

mod macros;
pub use macros::*;
