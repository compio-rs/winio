use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use smallvec::SmallVec;
use winio_layout::Layoutable;
use winio_primitive::{Point, Rect, Size};

use super::ComponentMessage;
use crate::{Component, ComponentSender};

/// Helper to embed one component into another. It handles different types of
/// messages and events.
pub struct Child<T: Component> {
    model: T,
    sender: ComponentSender<T>,
    msg_cache: SmallVec<[T::Message; 1]>,
}

impl<T: Component> Child<T> {
    /// Create and initialize the child component.
    pub fn init<'a>(init: impl Into<T::Init<'a>>) -> Self {
        let sender = ComponentSender::new();
        let model = T::init(init.into(), &sender);
        Self {
            model,
            sender,
            msg_cache: SmallVec::new(),
        }
    }

    /// Start to receive and interp the events of the child component.
    ///
    /// Define a root component `MainModel`, and it contains a
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
    ///     start! {
    ///         sender, default: MainMessage::Noop,
    ///         self.window => {
    ///             WindowEvent::Close => MainMessage::Close,
    ///         },
    ///         // ...other children
    ///     }
    /// }
    /// ```
    /// It is equivalent to
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
    ) -> ! {
        let fut_start = self.model.start(&self.sender);
        let fut_forward = async {
            loop {
                self.sender.wait().await;
                for msg in self.sender.fetch_all() {
                    match msg {
                        ComponentMessage::Message(msg) => {
                            self.msg_cache.push(msg);
                            sender.post(propagate());
                        }
                        ComponentMessage::Event(e) => {
                            if let Some(m) = f(e) {
                                sender.post(m);
                            }
                        }
                    }
                }
            }
        };
        futures_util::future::join(fut_start, fut_forward).await.0
    }

    /// Emit message to the child component.
    pub async fn emit(&mut self, message: T::Message) -> bool {
        self.model.update(message, &self.sender).await
    }

    /// Respond to the child message.
    pub async fn update(&mut self) -> bool {
        let mut need_render = false;
        for message in self.msg_cache.drain(..) {
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

impl<T: AsRawWindow + Component> AsRawWindow for Child<T> {
    fn as_raw_window(&self) -> RawWindow {
        self.model.as_raw_window()
    }
}

impl<T: AsWindow + Component> AsWindow for Child<T> {
    fn as_window(&self) -> BorrowedWindow<'_> {
        self.model.as_window()
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

    fn min_size(&self) -> Size {
        self.model.min_size()
    }
}
