mod action;
pub(crate) use action::*;

mod ffi;

mod bar;
pub use bar::*;

mod bar_item;
pub use bar_item::*;

mod item;
pub use item::*;

mod popup;
pub use popup::*;

mod check;
pub use check::*;

mod radio;
pub use radio::*;

mod separator;
pub use separator::*;

struct RawPtr<T>(*mut T);

impl<T> std::fmt::Debug for RawPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RawPtr").field(&self.0).finish()
    }
}

impl<T> RawPtr<T> {
    pub unsafe fn as_ref(&self) -> &T {
        unsafe { &*self.0 }
    }

    pub unsafe fn pin_mut(&mut self) -> std::pin::Pin<&mut T> {
        unsafe { std::pin::Pin::new_unchecked(&mut *self.0) }
    }
}
