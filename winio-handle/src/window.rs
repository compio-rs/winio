use std::{marker::PhantomData, ops::Deref};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        /// Raw window handle.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum RawWindow {
            /// Win32 `HWND`.
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND),
            /// WinUI `Window`.
            #[cfg(feature = "winui")]
            WinUI(winui3::Microsoft::UI::Xaml::Window),
        }
    } else if #[cfg(target_os = "macos")] {
        /// [`NSWindow`].
        ///
        /// [`NSWindow`]: objc2_app_kit::NSWindow
        pub type RawWindow = objc2::rc::Retained<objc2_app_kit::NSWindow>;
    } else {
        /// Raw window handle.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum RawWindow {
            /// Pointer to `QWidget`.
            #[cfg(all(not(any(windows, target_os = "macos")), feature = "qt"))]
            Qt(*mut core::ffi::c_void),
            /// GTK [`Window`].
            ///
            /// [`Window`]: gtk4::Window
            #[cfg(all(not(any(windows, target_os = "macos")), feature = "gtk"))]
            Gtk(gtk4::Window),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl RawWindow {
    /// Get Win32 `HWND`.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::Foundation::HWND {
        match self {
            Self::Win32(hwnd) => *hwnd,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get WinUI `Window`.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &winui3::Microsoft::UI::Xaml::Window {
        match self {
            Self::WinUI(window) => window,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_os = "macos")))]
impl RawWindow {
    /// Get Qt `QWidget`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match self {
            Self::Qt(w) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `Window`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> gtk4::Window {
        match self {
            Self::Gtk(w) => w.clone(),
            _ => panic!("unsupported handle type"),
        }
    }
}

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

impl Deref for BorrowedWindow<'_> {
    type Target = RawWindow;

    fn deref(&self) -> &Self::Target {
        &self.handle
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
