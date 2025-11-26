#[doc(hidden)]
pub use futures_util::{TryFutureExt as __TryFutureExt, join as __join, try_join as __try_join};
#[doc(hidden)]
pub use paste::paste as __paste;
#[doc(hidden)]
pub use tuplex::IntoArray as __IntoArray;

/// Helper macro for `Component::init`.
///
/// ```ignore
/// # use winio::prelude::*;
/// init! {
///     window: Window = (()) => {
///         text: "Basic example",
///         size: Size::new(800.0, 600.0),
///     },
///     canvas: Canvas = (&window),
/// }
/// window.show();
/// ```
#[macro_export]
macro_rules! init {
    () => {};
    ($($(#[$m:meta])* $name:ident : $t:ty = ($init:expr) $(=> { $($a:tt)* } )?),+$(,)?) => {
        $(
            #[allow(unused_mut)]
            $(#[$m])*
            let mut $name = $crate::Child::<$t>::init($init)?;
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

/// Helper macro for `Component::start`.
///
/// ```ignore
/// # use winio::prelude::*;
/// struct MainModel {
///     window: Child<Window>,
///     canvas: Child<Canvas>,
/// }
/// enum MainMessage {
///     Noop,
///     Redraw,
///     Close,
/// }
/// # impl Component for MainModel {
/// # type Init<'a> = (); type Message = MainMessage; type Event = ();
/// # fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self { todo!() }
/// # async fn update(&mut self, _msg: Self::Message, _sender: &ComponentSender<Self>) -> bool { false }
/// # fn render(&mut self, _sender: &ComponentSender<Self>) {}
/// async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
///     start! {
///         sender,
///         default: MainMessage::Noop,
///         self.window => {
///             WindowEvent::Close => MainMessage::Close,
///             WindowEvent::Resize => MainMessage::Redraw,
///         },
///         self.canvas => {
///             CanvasEvent::MouseMove(_) => MainMessage::Redraw,
///         },
///     };
/// }
/// # }
/// ```
#[macro_export]
macro_rules! start {
    ($sender:expr, default: $noop:expr $(,)?) => {
        let _sender = $sender;
        let _default = $noop;
        ::core::future::pending().await
    };
    ($sender:expr, default: $noop:expr, $($(#[$m:meta])* $w:expr => { $($t:tt)* }),+$(,)?) => {
        #[allow(unreachable_code)]
        $crate::__join!($(
            $(#[$m])*
            $w.start(
                $sender,
                $crate::__start_map!($($t)*),
                || $noop
            ),
        )*).0;
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

/// Helper macro for `Component::update` to update multiple children.
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
