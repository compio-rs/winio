#[doc(hidden)]
pub use futures_util::join as __join;
#[doc(hidden)]
pub use paste::paste as __paste;

/// Helper macro for `Component::init`.
///
/// ```no_run
/// # use winio::{Canvas, Layoutable, Size, Visible, Window, init};
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
    ($($name:ident : $t:ty = ($init:expr) $(=> { $($a:tt)* } )?),+$(,)?) => {
        $(
            #[allow(unused_mut)]
            let mut $name = $crate::Child::<$t>::init($init);
            $(
                $crate::__init_assign!($name, $($a)*);
            )?
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
                $name.[<set_ $prop>]($value);
            }
        )*
    };
}

/// Helper macro for `Component::start`.
///
/// ```no_run
/// # use winio::{Canvas, CanvasEvent, Child, Component, ComponentSender, Window, WindowEvent, start};
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

/// Helper macro for layouts in `Component::render`.
///
/// ```no_run
/// # use winio::{Canvas, Child, Grid, Layoutable, Window, layout};
/// # struct MainModel {
/// #     window: Child<Window>,
/// #     canvas: Child<Canvas>,
/// # }
/// # impl MainModel { fn foo(&mut self) {
/// let csize = self.window.client_size();
/// {
///     let mut grid = layout! {
///         Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap(),
///         self.canvas => { column: 1, row: 1 },
///     };
///     grid.set_size(csize);
/// }
/// # } }
/// ```
#[macro_export]
macro_rules! layout {
    ($root:expr, $($e:expr $(=>  { $($t:tt)* })?),+$(,)?) => {{
        #[allow(unused_mut)]
        let mut root = $root;
        $(
            $crate::__layout_push!(root, &mut $e, $($($t)*)?);
        )+
        root
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __layout_push {
    ($root:expr, $e:expr,) => {
        $root.push($e).finish();
    };
    ($root:expr, $e:expr, $($(#[$me:meta])* $p:ident : $v:expr),+$(,)?) => {
        let builder = $root.push($e);
        $(
            $(#[$me])*
            let builder = builder.$p($v);
        )+
        builder.finish();
    };
}
