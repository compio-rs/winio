use std::{future::Future, hint::unreachable_unchecked};

use futures_util::FutureExt;

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
    async fn start(&mut self, sender: &ComponentSender<Self>) -> !;

    /// Respond to the message.
    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool;

    /// Render the widgets.
    fn render(&mut self, sender: &ComponentSender<Self>);
}

#[derive(Debug)]
enum ComponentMessage<T: Component> {
    Message(T::Message),
    Event(T::Event),
}

/// Sender of input messages and output events.
#[derive(Debug)]
pub struct ComponentSender<T: Component>(Sender<ComponentMessage<T>>);

impl<T: Component> Clone for ComponentSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug)]
struct ComponentReceiver<T: Component>(Receiver<ComponentMessage<T>>);

fn component_channel<T: Component>() -> (ComponentSender<T>, ComponentReceiver<T>) {
    let (tx, rx) = channel();
    (ComponentSender(tx), ComponentReceiver(rx))
}

impl<T: Component> ComponentSender<T> {
    /// Post the message to the queue.
    pub fn post(&self, message: T::Message) {
        self.0.send(ComponentMessage::Message(message))
    }

    /// Post the event to the queue.
    pub fn output(&self, event: T::Event) {
        self.0.send(ComponentMessage::Event(event))
    }
}

impl<T: Component> ComponentReceiver<T> {
    async fn wait(&self) {
        self.0.wait().await
    }

    fn fetch_all(&self) -> Vec<ComponentMessage<T>> {
        self.0.fetch_all()
    }
}

/// Root application, manages the async runtime.
pub struct App {
    runtime: Runtime,
    name: Option<String>,
}

impl App {
    /// Create [`App`] with application name.
    pub fn new(name: impl AsRef<str>) -> Self {
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
            let (sender, recv) = component_channel();
            let mut model = T::init(init.into(), &sender);
            model.render(&sender);
            'outer: loop {
                let fut_start = model.start(&sender);
                let fut_recv = recv.wait();
                futures_util::select! {
                    // SAFETY: never type
                    _ = fut_start.fuse() => unsafe { unreachable_unchecked() },
                    _ = fut_recv.fuse() => {
                        let mut need_render = false;
                        for msg in recv.fetch_all() {
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

mod channel;
pub(crate) use channel::*;

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

mod list_box;
pub use list_box::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;

mod macros;
pub use macros::*;
