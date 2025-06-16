use std::marker::PhantomData;

use crate::ui::{RawWindow, Window};

/// A borrowed window handle.
#[derive(Clone)]
pub struct BorrowedWindow<'a> {
    handle: RawWindow,
    _p: PhantomData<&'a ()>,
}

impl BorrowedWindow<'_> {
    /// # Safety
    ///
    /// The window must remain valid for the duration of the returned
    /// [`BorrowedWindow`].
    pub unsafe fn borrow_raw(handle: RawWindow) -> Self {
        Self {
            handle,
            _p: PhantomData,
        }
    }
}

/// Trait to exact the raw window handle.
pub trait AsRawWindow {
    /// Get the raw window handle.
    fn as_raw_window(&self) -> RawWindow;
}

impl AsRawWindow for RawWindow {
    #[allow(clippy::clone_on_copy)]
    fn as_raw_window(&self) -> RawWindow {
        self.clone()
    }
}

impl AsRawWindow for BorrowedWindow<'_> {
    #[allow(clippy::clone_on_copy)]
    fn as_raw_window(&self) -> RawWindow {
        self.handle.clone()
    }
}

impl<T: AsRawWindow> AsRawWindow for &'_ T {
    fn as_raw_window(&self) -> RawWindow {
        (**self).as_raw_window()
    }
}

/// Trait to borrow the window handle.
pub trait AsWindow {
    /// Get the window handle.
    fn as_window(&self) -> BorrowedWindow<'_>;
}

impl AsWindow for BorrowedWindow<'_> {
    fn as_window(&self) -> BorrowedWindow<'_> {
        self.clone()
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.as_raw_window()) }
    }
}

impl<T: AsWindow + ?Sized> AsWindow for &T {
    #[inline]
    fn as_window(&self) -> BorrowedWindow<'_> {
        T::as_window(self)
    }
}

impl<'a, T: AsWindow + ?Sized> From<&'a T> for BorrowedWindow<'a> {
    fn from(value: &'a T) -> Self {
        value.as_window()
    }
}

#[doc(hidden)]
pub struct MaybeBorrowedWindow<'a>(pub Option<BorrowedWindow<'a>>);

impl<'a, T: Into<BorrowedWindow<'a>>> From<T> for MaybeBorrowedWindow<'a> {
    fn from(value: T) -> Self {
        Self(Some(value.into()))
    }
}

impl<'a, T: Into<BorrowedWindow<'a>>> From<Option<T>> for MaybeBorrowedWindow<'a> {
    fn from(value: Option<T>) -> Self {
        Self(value.map(|v| v.into()))
    }
}

impl From<()> for MaybeBorrowedWindow<'_> {
    fn from(_: ()) -> Self {
        Self(None)
    }
}
