use std::ops::{Deref, DerefMut};

use crate::{Component, ComponentSender};

pub struct Child<T: Component> {
    model: T,
    sender: ComponentSender<T>,
}

impl<T: Component> Child<T> {
    pub fn init(counter: T::Init, root: &T::Root) -> Self {
        let sender = ComponentSender::new();
        let model = T::init(counter, root, sender.clone());
        Self {
            model,
            sender: sender.clone(),
        }
    }

    pub async fn start<C: Component>(
        &mut self,
        sender: ComponentSender<C>,
        mut f: impl FnMut(T::Event) -> Option<C::Message>,
    ) {
        let fut_start = self.model.start(self.sender.clone());
        let fut_forward = async {
            loop {
                let e = self.sender.recv_output().await;
                if let Some(m) = f(e) {
                    sender.post(m).await;
                }
            }
        };
        futures_util::future::join(fut_start, fut_forward).await;
    }

    pub async fn update(&mut self, message: T::Message) -> bool {
        self.model.update(message, self.sender.clone()).await
    }

    pub fn render(&mut self) {
        self.model.render(self.sender.clone());
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
