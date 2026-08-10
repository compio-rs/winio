#[doc(hidden)]
pub use futures_util::{TryFutureExt as __TryFutureExt, join as __join, try_join as __try_join};
#[doc(hidden)]
pub use paste::paste as __paste;
#[doc(hidden)]
pub use tuplex::IntoArray as __IntoArray;

/// Helper macro for [`Component::init`](crate::Component::init) to initialize
/// the child components.
///
/// It creates the child components with [`Child::init`](crate::Child::init),
/// and applies the initial properties to them. It is the recommended way to
/// initialize the children in [`Component::init`](crate::Component::init).
///
/// Each entry has the form:
///
/// ```text
/// name: Type = (init),
/// name: Type = (init) => { property: value, ... },
/// ```
///
/// The second form additionally applies the initial properties. The first
/// form is a shorthand when no property needs to be set.
///
/// * `name`: the field name of the child component in the parent;
/// * `Type`: the type of the child component;
/// * `init`: the initial parameters passed to
///   [`Component::init`](crate::Component::init);
/// * `property: value`: the initial properties, applied with
///   `set_property(value)` on the child.
///
/// The entry, as well as each property, may be prefixed with attributes,
/// e.g. to apply a property only on a certain platform:
///
/// ```text
/// #[cfg(windows)]
/// name: Type = (init) => {
///     #[cfg(win32)]
///     property: value,
/// },
/// ```
///
/// # Example
///
/// ```ignore
/// # use winio::prelude::*;
/// async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
///     init! {
///         window: Window = (()) => {
///             text: "Basic example",
///             size: Size::new(800.0, 600.0),
///         },
///         canvas: Canvas = (&window),
///         button: Button = (&window) => {
///             text: "Click me",
///             enabled: true,
///         },
///     }
///     Ok(Self {
///         window,
///         canvas,
///         button,
///     })
/// }
/// ```
///
/// The `window` and `canvas` variables are `Child<T>` instances that can be
/// stored in the component struct, and used in [`start!`](crate::start) and
/// [`update_children!`](crate::update_children).
#[macro_export]
macro_rules! init {
    () => {};
    ($($(#[$m:meta])* $name:ident : $t:ty = ($init:expr) $(=> { $($a:tt)* } )?),+$(,)?) => {
        $(
            #[allow(unused_mut)]
            $(#[$m])*
            let mut $name = $crate::Child::<$t>::init($init).await?;
            $(#[$m])*
            {
                $(
                    $crate::__init_assign!($name, $($a)*);
                )?
            }
        )*
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __init_assign {
    ($name:ident, ) => {};
    ($name:ident, $($(#[$m:meta])* $prop:ident : $value:expr),+$(,)?) => {
        $(
            $(#[$m])*
            $crate::__paste! {
                $name.[<set_ $prop>]($value)?;
            }
        )*
    };
}

/// Helper macro for [`Component::start`](crate::Component::start) to start the
/// child components.
///
/// It starts all the child components concurrently, and forwards the events
/// emitted by them to the parent component as messages.
///
/// Each entry has the form:
///
/// ```text
/// self.child => {
///     ChildEvent::Variant => ParentMessage::Variant,
///     ...
/// }
/// ```
///
/// * `self.child`: the child component field;
/// * `ChildEvent::Variant => ParentMessage::Variant`: maps an event emitted by
///   the child to a message of the parent. The events that are not listed are
///   ignored.
///
/// An empty mapping `{}` starts the child component without listening to any
/// of its events.
///
/// The mapping, may be prefixed with attributes, e.g. to map an event only on a
/// certain platform:
///
/// ```text
/// self.child => {
///     #[cfg(win32)]
///     ChildEvent::Variant => ParentMessage::Variant,
/// },
/// ```
///
/// # Example
///
/// ```ignore
/// # use winio::prelude::*;
/// struct MainModel {
///     window: Child<Window>,
///     canvas: Child<Canvas>,
/// }
/// enum MainMessage {
///     Redraw,
///     Close,
/// }
/// # impl Component for MainModel {
/// # type Init<'a> = (); type Message = MainMessage; type Event = (); type Error = Error;
/// # async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> { todo!() }
/// # async fn update(&mut self, _msg: Self::Message, _sender: &ComponentSender<Self>) -> Result<bool> { Ok(false) }
/// # fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> { Ok(()) }
/// async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
///     start! {
///         sender,
///
///         self.window => {
///             WindowEvent::Close => MainMessage::Close,
///             WindowEvent::Resize => MainMessage::Redraw,
///         },
///         self.canvas => {
///             CanvasEvent::MouseMove(_) => MainMessage::Redraw,
///         },
///     }
/// }
/// # }
/// ```
///
/// It is equivalent to calling [`Child::start`](crate::Child::start) on each
/// child and joining the futures together.
#[macro_export]
macro_rules! start {
    ($sender:expr $(,)?) => {
        let _sender = $sender;
        ::core::future::pending().await
    };
    ($sender:expr, $($(#[$m:meta])* $w:expr => { $($t:tt)* }),+$(,)?) => {
        #[allow(unreachable_code)]
        $crate::__join!($(
            $(#[$m])*
            $w.start(
                $sender,
                $crate::__start_map!($($t)*),
            ),
        )*).0
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __start_map {
    () => {
        |_| None
    };
    ($f:expr) => { $f };
    ($($(#[$me:meta])* $e:pat => $m:expr),+$(,)?) => {
        |e| match e {
            $(
                $(#[$me])*
                $e => Some($m),
            )*
            _ => None,
        }
    }
}

/// Helper macro for
/// [`Component::update_children`](crate::Component::update_children) to update
/// multiple children.
///
/// It calls [`Child::update`](crate::Child::update) on all the given children
/// concurrently, and returns `true` if any of them needs rendering.
///
/// # Example
///
/// ```ignore
/// async fn update_children(&mut self) -> Result<bool> {
///     update_children!(
///         self.window,
///         self.button,
///         self.label,
///     )
/// }
/// ```
///
/// It is equivalent to calling [`Child::update`](crate::Child::update) on each
/// child and joining the results together.
#[macro_export]
macro_rules! update_children {
    () => {
        $crate::try_join_update!()
    };
    ($c:expr) => {
        $crate::try_join_update!($c.update())
    };
    ($($c:expr),+$(,)?) => {
        $crate::try_join_update!($($c.update()),+)
    };
}

/// Helper macro for joining multiple update futures that return
/// [`Result<bool>`].
///
/// The futures are joined concurrently, and the returned values are
/// combined with a logical OR. The error types are converted with
/// [`From`], so the futures may have different error types as long as
/// they can be converted to the same one.
#[macro_export]
macro_rules! try_join_update {
    () => {
        Ok(false)
    };
    ($e:expr) => {
        Ok($e.await?)
    };
    ($($e:expr),+$(,)?) => {
        $crate::__try_join!($(
            $crate::__TryFutureExt::map_err($e, std::convert::From::from),
        )*).map(|res|{
            $crate::__IntoArray::into_array(res)
            .into_iter()
            .any(|b| b)
        })
    };
}
