use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "primitive")]
use inherit_methods_macro::inherit_methods;
use smallvec::SmallVec;
#[cfg(feature = "handle")]
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWidget, AsRawWindow, AsWidget, AsWindow, BorrowedContainer,
    BorrowedWidget, BorrowedWindow, RawContainer, RawWidget, RawWindow,
};
#[cfg(feature = "primitive")]
use winio_primitive::{Failable, Layoutable, Point, Rect, Size};

use super::ComponentMessage;
use crate::{Component, ComponentSender, Root};

/// Helper to embed one component into another. It handles different types of
/// messages and events.
pub struct Child<T: Component> {
    model: T,
    sender: ComponentSender<T>,
    msg_cache: SmallVec<[T::Message; 1]>,
}

impl<T: Component> Child<T> {
    /// Create and initialize the child component.
    pub fn init<'a>(init: impl Into<T::Init<'a>>) -> Result<Self, T::Error> {
        let sender = ComponentSender::new();
        let model = T::init(init.into(), &sender)?;
        Ok(Self::new(model, sender))
    }

    pub(crate) fn new(model: T, sender: ComponentSender<T>) -> Self {
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
    /// async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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
    /// async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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

    /// Post message to the child component.
    pub fn post(&mut self, message: T::Message) {
        self.sender.post(message);
    }

    /// Emit message to the child component.
    pub async fn emit(&mut self, message: T::Message) -> Result<bool, T::Error> {
        self.model.update(message, &self.sender).await
    }

    /// Respond to the child message.
    pub async fn update(&mut self) -> Result<bool, T::Error> {
        let mut need_render = self.model.update_children().await?;
        for message in self.msg_cache.drain(..) {
            need_render |= self.model.update(message, &self.sender).await?;
        }
        Ok(need_render)
    }

    /// Render the child component.
    pub fn render(&mut self) -> Result<(), T::Error> {
        self.model.render(&self.sender)?;
        self.model.render_children()
    }

    /// Get the sender of the child component.
    pub fn sender(&self) -> &ComponentSender<T> {
        &self.sender
    }

    /// Try to convert the child component into a root component.
    ///
    /// It clears the inner message cache and updates the child component if
    /// needed.
    pub async fn try_into_root(mut self) -> Result<Root<T>, T::Error> {
        if self.update().await? {
            self.render()?;
        }
        Ok(Root::new(self.model, self.sender))
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

#[cfg(feature = "handle")]
impl<T: AsRawWindow + Component> AsRawWindow for Child<T> {
    fn as_raw_window(&self) -> RawWindow {
        self.model.as_raw_window()
    }
}

#[cfg(feature = "handle")]
impl<T: AsWindow + Component> AsWindow for Child<T> {
    fn as_window(&self) -> BorrowedWindow<'_> {
        self.model.as_window()
    }
}

#[cfg(feature = "handle")]
impl<T: AsRawWidget + Component> AsRawWidget for Child<T> {
    fn as_raw_widget(&self) -> RawWidget {
        self.model.as_raw_widget()
    }
}

#[cfg(feature = "handle")]
impl<T: AsWidget + Component> AsWidget for Child<T> {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        self.model.as_widget()
    }
}

#[cfg(feature = "handle")]
impl<T: AsRawContainer + Component> AsRawContainer for Child<T> {
    fn as_raw_container(&self) -> RawContainer {
        self.model.as_raw_container()
    }
}

#[cfg(feature = "handle")]
impl<T: AsContainer + Component> AsContainer for Child<T> {
    fn as_container(&self) -> BorrowedContainer<'_> {
        self.model.as_container()
    }
}

impl<T: Component + Debug> Debug for Child<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Child").field("model", &self.model).finish()
    }
}

#[cfg(feature = "primitive")]
impl<T: Component + Failable> Failable for Child<T> {
    type Error = <T as Failable>::Error;
}

#[cfg(feature = "primitive")]
#[inherit_methods(from = "self.model")]
impl<T: Component + Layoutable> Layoutable for Child<T> {
    fn loc(&self) -> Result<Point, Self::Error>;

    fn set_loc(&mut self, p: Point) -> Result<(), Self::Error>;

    fn size(&self) -> Result<Size, Self::Error>;

    fn set_size(&mut self, s: Size) -> Result<(), Self::Error>;

    fn rect(&self) -> Result<Rect, Self::Error>;

    fn set_rect(&mut self, r: Rect) -> Result<(), Self::Error>;

    fn preferred_size(&self) -> Result<Size, Self::Error>;

    fn min_size(&self) -> Result<Size, Self::Error>;
}
