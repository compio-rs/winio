use std::{convert::Infallible, fmt::Debug, pin::Pin};

use futures_util::FutureExt;

use crate::{Component, ComponentMessage, ComponentSender, channel::Channel};

type DynComponentSender<T> =
    Channel<ComponentMessage<<T as DynComponent>::Message, <T as DynComponent>::Event>>;

trait DynComponent {
    type Message;
    type Event;
    type Error: Debug;

    fn start<'a>(
        &'a mut self,
        sender: &'a DynComponentSender<Self>,
    ) -> Pin<Box<dyn Future<Output = Infallible> + 'a>>;

    fn update_children(&mut self) -> Pin<Box<dyn Future<Output = Result<bool, Self::Error>> + '_>>;

    fn update<'a>(
        &'a mut self,
        message: Self::Message,
        sender: &'a DynComponentSender<Self>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Self::Error>> + 'a>>;

    fn render(&mut self, sender: &DynComponentSender<Self>) -> Result<(), Self::Error>;

    fn render_children(&mut self) -> Result<(), Self::Error>;
}

impl<T: Component + 'static> DynComponent for T {
    type Error = <T as Component>::Error;
    type Event = <T as Component>::Event;
    type Message = <T as Component>::Message;

    fn start<'a>(
        &'a mut self,
        sender: &'a DynComponentSender<Self>,
    ) -> Pin<Box<dyn Future<Output = Infallible> + 'a>> {
        let sender = ComponentSender::<T>::from_ref(sender);
        Box::pin(Component::start(self, sender).map(|a| a))
    }

    fn update_children(&mut self) -> Pin<Box<dyn Future<Output = Result<bool, Self::Error>> + '_>> {
        Box::pin(Component::update_children(self))
    }

    fn update<'a>(
        &'a mut self,
        message: Self::Message,
        sender: &'a DynComponentSender<Self>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Self::Error>> + 'a>> {
        let sender = ComponentSender::<T>::from_ref(sender);
        Box::pin(Component::update(self, message, sender))
    }

    fn render(&mut self, sender: &DynComponentSender<Self>) -> Result<(), Self::Error> {
        let sender = ComponentSender::<T>::from_ref(sender);
        Component::render(self, sender)
    }

    fn render_children(&mut self) -> Result<(), Self::Error> {
        Component::render_children(self)
    }
}

/// A boxed component. It is not initialized directly, but constructed by
/// [`Child::into_boxed`] or [`Root::into_boxed`].
///
/// [`Child::into_boxed`]: crate::Child::into_boxed
/// [`Root::into_boxed`]: crate::Root::into_boxed
pub struct BoxComponent<M, Ev, Err>(Box<dyn DynComponent<Message = M, Event = Ev, Error = Err>>);

impl<M, Ev, Err> BoxComponent<M, Ev, Err> {
    pub(crate) fn new<T: Component<Message = M, Event = Ev, Error = Err> + 'static>(
        component: T,
    ) -> Self {
        Self(Box::new(component))
    }
}

impl<M, Ev, Err: Debug> Component for BoxComponent<M, Ev, Err> {
    type Error = Err;
    type Event = Ev;
    type Init<'a> = Infallible;
    type Message = M;

    async fn init(
        init: Self::Init<'_>,
        _sender: &ComponentSender<Self>,
    ) -> Result<Self, Self::Error> {
        match init {}
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let sender = &sender.0;
        match self.0.start(sender).await {}
    }

    async fn update_children(&mut self) -> Result<bool, Self::Error> {
        self.0.update_children().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool, Self::Error> {
        let sender = &sender.0;
        self.0.update(message, sender).await
    }

    fn render(&mut self, sender: &ComponentSender<Self>) -> Result<(), Self::Error> {
        let sender = &sender.0;
        self.0.render(sender)
    }

    fn render_children(&mut self) -> Result<(), Self::Error> {
        self.0.render_children()
    }
}

impl<M, Ev, Err> Debug for BoxComponent<M, Ev, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxComponent").finish_non_exhaustive()
    }
}
