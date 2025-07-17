use std::{marker::PhantomData, ops::Deref};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        /// Raw window handle.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum RawWidget {
            /// Win32 `HWND`.
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND),
            /// WinUI `FrameworkElement`.
            #[cfg(feature = "winui")]
            WinUI(winui3::Microsoft::UI::Xaml::FrameworkElement),
        }
    } else if #[cfg(target_os = "macos")] {
        /// [`NSView`].
        ///
        /// [`NSView`]: objc2_app_kit::NSView
        pub type RawWidget = objc2::rc::Retained<objc2_app_kit::NSView>;
    } else {
        /// Raw window handle.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum RawWidget {
            /// Pointer to `QWidget`.
            #[cfg(all(not(any(windows, target_os = "macos")), feature = "qt"))]
            Qt(*mut core::ffi::c_void),
            /// GTK [`Widget`].
            ///
            /// [`Widget`]: gtk4::Widget
            #[cfg(all(not(any(windows, target_os = "macos")), feature = "gtk"))]
            Gtk(gtk4::Widget),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl RawWidget {
    /// Get Win32 `HWND`.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::Foundation::HWND {
        match self {
            Self::Win32(hwnd) => *hwnd,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get WinUI `FrameworkElement`.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &winui3::Microsoft::UI::Xaml::FrameworkElement {
        match self {
            Self::WinUI(e) => e,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_os = "macos")))]
impl RawWidget {
    /// Get Qt `QWidget`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match self {
            Self::Qt(w) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `Widget`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> gtk4::Widget {
        match self {
            Self::Gtk(w) => w.clone(),
            _ => panic!("unsupported handle type"),
        }
    }
}

/// A borrowed window handle.
#[derive(Clone)]
pub struct BorrowedWidget<'a> {
    handle: RawWidget,
    _p: PhantomData<&'a ()>,
}

impl BorrowedWidget<'_> {
    /// # Safety
    ///
    /// The window must remain valid for the duration of the returned
    /// [`BorrowedWidget`].
    pub unsafe fn borrow_raw(handle: RawWidget) -> Self {
        Self {
            handle,
            _p: PhantomData,
        }
    }
}

impl Deref for BorrowedWidget<'_> {
    type Target = RawWidget;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Trait to exact the raw window handle.
pub trait AsRawWidget {
    /// Get the raw window handle.
    fn as_raw_widget(&self) -> RawWidget;
}

impl AsRawWidget for RawWidget {
    #[allow(clippy::clone_on_copy)]
    fn as_raw_widget(&self) -> RawWidget {
        self.clone()
    }
}

impl AsRawWidget for BorrowedWidget<'_> {
    #[allow(clippy::clone_on_copy)]
    fn as_raw_widget(&self) -> RawWidget {
        self.handle.clone()
    }
}

impl<T: AsRawWidget> AsRawWidget for &'_ T {
    fn as_raw_widget(&self) -> RawWidget {
        (**self).as_raw_widget()
    }
}

/// Trait to borrow the window handle.
pub trait AsWidget {
    /// Get the window handle.
    fn as_widget(&self) -> BorrowedWidget<'_>;
}

impl AsWidget for BorrowedWidget<'_> {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        self.clone()
    }
}

impl<T: AsWidget + ?Sized> AsWidget for &T {
    #[inline]
    fn as_widget(&self) -> BorrowedWidget<'_> {
        T::as_widget(self)
    }
}

impl<'a, T: AsWidget + ?Sized> From<&'a T> for BorrowedWidget<'a> {
    fn from(value: &'a T) -> Self {
        value.as_widget()
    }
}
