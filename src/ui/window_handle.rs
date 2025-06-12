use std::marker::PhantomData;

use crate::ui::RawWindow;

/// A borrowed window handle.
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
