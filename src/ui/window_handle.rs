use std::marker::PhantomData;

use crate::ui::RawWindow;

pub struct BorrowedWindow<'a> {
    handle: RawWindow,
    _p: PhantomData<&'a ()>,
}

impl BorrowedWindow<'_> {
    /// # Safety
    ///
    /// The window must remain valid for the duration of the returned
    /// [`BorrowedWindow`].
    pub unsafe fn from_raw(handle: RawWindow) -> Self {
        Self {
            handle,
            _p: PhantomData,
        }
    }
}

pub trait AsRawWindow {
    fn as_raw_window(&self) -> RawWindow;
}

impl AsRawWindow for RawWindow {
    fn as_raw_window(&self) -> RawWindow {
        self.clone()
    }
}

impl AsRawWindow for BorrowedWindow<'_> {
    fn as_raw_window(&self) -> RawWindow {
        self.handle.clone()
    }
}

impl<T: AsRawWindow> AsRawWindow for &'_ T {
    fn as_raw_window(&self) -> RawWindow {
        (**self).as_raw_window()
    }
}

pub trait AsWindow {
    fn as_window(&self) -> BorrowedWindow<'_>;
}

impl<T: AsRawWindow> AsWindow for T {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::from_raw(self.as_raw_window()) }
    }
}
