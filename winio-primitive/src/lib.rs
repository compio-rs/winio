//! Primitive types for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

mod drawing;
pub use drawing::*;

mod monitor;
pub use monitor::*;

mod canvas;
pub use canvas::*;

mod msgbox;
pub use msgbox::*;

#[doc(hidden)]
#[macro_export]
macro_rules! inherit {
    ($base:ident, $($(#[$attr:meta])* $v:vis fn $name:ident($($t:tt)*) $(-> $ret:ty)?;)*) => {
        $(
            $(#[$attr])* $v fn $name($($t)*) $(-> $ret)? {
                $crate::__inherit_call!($base, $name, $($t)*)
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __inherit_call {
    ($base:ident, $name:ident, & $this:ident $(, $arg:ident: $argt:ty)*$(,)?) => {
        $this.$base.$name($($arg, )*)
    };
    ($base:ident, $name:ident, &mut $this:ident $(, $arg:ident: $argt:ty)*$(,)?) => {
        $this.$base.$name($($arg, )*)
    };
}
