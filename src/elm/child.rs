use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{ComponentReceiver, component_channel};
use crate::{Component, ComponentSender};

/// Helper to embed one component into another. It handles different types of
/// messages and events.
pub struct Child<T: Component> {
    model: T,
    sender: ComponentSender<T>,
    receiver: ComponentReceiver<T>,
}

impl<T: Component> Child<T> {
    /// Create and initialize the child component.
    pub fn init(counter: T::Init, root: &T::Root) -> Self {
        let (sender, receiver) = component_channel();
        let model = T::init(counter, root, &sender);
        Self {
            model,
            sender,
            receiver,
        }
    }

    /// Start to receive and interp the events of the child component.
    pub async fn start<C: Component>(
        &mut self,
        sender: &ComponentSender<C>,
        mut f: impl FnMut(T::Event) -> Option<C::Message>,
    ) {
        let fut_start = self.model.start(&self.sender);
        let fut_forward = async {
            loop {
                let e = self.receiver.recv_output().await;
                if let Some(m) = f(e) {
                    sender.post(m);
                }
            }
        };
        futures_util::future::join(fut_start, fut_forward).await;
    }

    /// Emit message to the child component.
    pub async fn emit(&mut self, message: T::Message) -> bool {
        self.model.update(message, &self.sender).await
    }

    /// Respond to the child message.
    pub async fn update(&mut self) -> bool {
        let mut need_render = false;
        while let Some(message) = self.receiver.try_recv() {
            need_render |= self.model.update(message, &self.sender).await;
        }
        need_render
    }

    /// Render the child component.
    pub fn render(&mut self) {
        self.model.render(&self.sender);
    }
}

impl<T: Component> Deref for Child<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<T: Component> DerefMut for Child<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

impl<T: Component + Debug> Debug for Child<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Child").field("model", &self.model).finish()
    }
}
