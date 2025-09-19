mod common;
pub(crate) use common::*;

mod window;
pub use window::*;

mod canvas;
pub use canvas::*;

mod widget;
pub use widget::View;
pub(crate) use widget::*;

mod monitor;
pub use monitor::*;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod scroll_bar;
pub use scroll_bar::*;

mod scroll_view;
pub use scroll_view::*;

#[cfg(feature = "media")]
mod media;
#[cfg(feature = "media")]
pub use media::*;

#[cfg(feature = "webview")]
mod webview;
#[cfg(feature = "webview")]
pub use webview::*;

mod tooltip;
pub use tooltip::*;

#[doc(hidden)]
pub trait StaticCastTo<T> {
    fn static_cast(&self) -> &T;
    fn static_cast_mut(self: Pin<&mut Self>) -> Pin<&mut T>;
}

impl<T> StaticCastTo<T> for T {
    fn static_cast(&self) -> &T {
        self
    }

    fn static_cast_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        self
    }
}

macro_rules! impl_static_cast {
    ($t:ty, $tbase:ty) => {
        impl $crate::ui::StaticCastTo<$tbase> for $t {
            fn static_cast(&self) -> &$tbase {
                unsafe { &*(self as *const Self).cast() }
            }

            fn static_cast_mut(self: ::std::pin::Pin<&mut Self>) -> ::std::pin::Pin<&mut $tbase> {
                unsafe {
                    ::std::pin::Pin::new_unchecked(
                        &mut *(self.get_unchecked_mut() as *mut Self).cast(),
                    )
                }
            }
        }
    };
}

macro_rules! impl_static_cast_propogate {
    ($t1:ty, $t2:ty, $t3:ty) => {
        impl $crate::ui::StaticCastTo<$t3> for $t1 {
            fn static_cast(&self) -> &$t3 {
                <Self as $crate::ui::StaticCastTo<$t2>>::static_cast(self).static_cast()
            }

            fn static_cast_mut(self: ::std::pin::Pin<&mut Self>) -> ::std::pin::Pin<&mut $t3> {
                <Self as $crate::ui::StaticCastTo<$t2>>::static_cast_mut(self).static_cast_mut()
            }
        }
    };
}

pub(crate) use impl_static_cast;
pub(crate) use impl_static_cast_propogate;

#[inline(always)]
pub(crate) fn static_cast<T>(p: &impl StaticCastTo<T>) -> &T {
    StaticCastTo::<T>::static_cast(p)
}

#[inline(always)]
pub(crate) fn static_cast_mut<T>(p: Pin<&mut impl StaticCastTo<T>>) -> Pin<&mut T> {
    StaticCastTo::<T>::static_cast_mut(p)
}

use std::pin::Pin;

use winio_primitive::ColorTheme;

pub fn color_theme() -> ColorTheme {
    if is_dark() {
        ColorTheme::Dark
    } else {
        ColorTheme::Light
    }
}
