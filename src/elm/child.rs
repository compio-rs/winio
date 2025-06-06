use std::{
    collections::VecDeque,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{ComponentReceiver, Layoutable, component_channel};
use crate::{Component, ComponentSender, Point, Rect, Size};

/// Helper to embed one component into another. It handles different types of
/// messages and events.
pub struct Child<T: Component> {
    model: T,
    sender: ComponentSender<T>,
    msg: ComponentReceiver<T::Message>,
    ev: ComponentReceiver<T::Event>,
    msg_cache: VecDeque<T::Message>,
}

impl<T: Component> Child<T> {
    /// Create and initialize the child component.
    pub fn init(init: T::Init, root: &T::Root) -> Self {
        let (sender, msg, ev) = component_channel();
        let model = T::init(init, root, &sender);
        Self {
            model,
            sender,
            msg,
            ev,
            msg_cache: VecDeque::new(),
        }
    }

    /// Start to receive and interp the events of the child component.
    ///
    /// Let's support there's a root component `MainModel`, and it contains a
    /// `window: Child<Window>`. The message of `MainModel` is defined as
    /// ```ignore
    /// enum MainMessage {
    ///     Noop,
    ///     Close,
    /// }
    /// ```
    /// In the `MainModel::start`, you should write
    /// ```ignore
    /// async fn start(&mut self, sender: &ComponentSender<Self>) {
    ///     let fut_window = self.window.start(
    ///         sender,
    ///         |e| match e {
    ///             WindowEvent::Close => Some(MainMessage::Close),
    ///             // ignore other events
    ///             _ => None,
    ///         },
    ///         // you should always propagate internal messages
    ///         || MainMessage::Noop,
    ///     );
    ///     // ...other children
    ///     futures_util::join!(fut_window, /* ... */);
    /// }
    /// ```
    pub async fn start<C: Component>(
        &mut self,
        sender: &ComponentSender<C>,
        mut f: impl FnMut(T::Event) -> Option<C::Message>,
        mut propagate: impl FnMut() -> C::Message,
    ) {
        let fut_start = self.model.start(&self.sender);
        let fut_forward = async {
            loop {
                let e = self.ev.recv().await;
                if let Some(m) = f(e) {
                    sender.post(m);
                }
            }
        };
        let fut_internal = async {
            loop {
                let e = self.msg.recv().await;
                self.msg_cache.push_back(e);
                sender.post(propagate());
            }
        };
        futures_util::future::join3(fut_start, fut_forward, fut_internal).await;
    }

    /// Emit message to the child component.
    pub async fn emit(&mut self, message: T::Message) -> bool {
        self.model.update(message, &self.sender).await
    }

    /// Respond to the child message.
    pub async fn update(&mut self) -> bool {
        let mut need_render = false;
        while let Some(message) = self.msg_cache.pop_front() {
            need_render |= self.model.update(message, &self.sender).await;
        }
        while let Some(message) = self.msg.try_recv() {
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

impl<T: Component + Layoutable> Layoutable for Child<T> {
    fn loc(&self) -> Point {
        self.model.loc()
    }

    fn set_loc(&mut self, p: Point) {
        self.model.set_loc(p);
    }

    fn size(&self) -> Size {
        self.model.size()
    }

    fn set_size(&mut self, s: Size) {
        self.model.set_size(s);
    }

    fn rect(&self) -> Rect {
        self.model.rect()
    }

    fn set_rect(&mut self, r: Rect) {
        self.model.set_rect(r);
    }

    fn preferred_size(&self) -> Size {
        self.model.preferred_size()
    }
}
