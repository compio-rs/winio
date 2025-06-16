#[doc(hidden)]
pub use futures_util::join as __join;
#[doc(hidden)]
pub use paste::paste as __paste;

/// Helper macro for `Component::init`.
///
/// ```ignore
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
    ($($name:ident : $t:ty = ($init:expr) $(=> { $($prop:ident : $value:expr),*$(,)? } )?),*$(,)?) => {
        $(
            #[allow(unused_mut)]
            let mut $name = $crate::Child::<$t>::init($init);
            $(
                $(
                    $crate::__paste! {
                        $name.[<set_ $prop>]($value);
                    }
                )*
            )?
        )*
    };
}

/// Helper macro for `Component::start`.
///
/// ```ignore
/// start! {
///     sender,
///     default: MainMessage::Noop,
///     self.window => {
///         WindowEvent::Close => MainMessage::Close,
///         WindowEvent::Resize => MainMessage::Redraw,
///     },
///     self.canvas => {
///         CanvasEvent::Redraw => MainMessage::Redraw,
///     },
/// };
/// ```
#[macro_export]
macro_rules! start {
    ($sender:expr, default: $noop:expr, $($w:expr =>  { $($e:pat => $m:expr),*$(,)? }),*$(,)?) => {
        $crate::__join!($(
            $w.start(
                $sender,
                |e| match e {
                    $(
                        $e => Some($m),
                    )*
                    _ => None,
                },
                || $noop
            ),
        )*);
    };
}
