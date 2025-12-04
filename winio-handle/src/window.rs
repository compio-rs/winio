#[cfg(feature = "raw-window-handle")]
use raw_window_handle::{HandleError, HasWindowHandle, WindowHandle};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        enum BorrowedWindowInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND, PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::Window),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
    } else if #[cfg(target_os = "macos")] {
        /// [`NSWindow`].
        ///
        /// [`NSWindow`]: objc2_app_kit::NSWindow
        pub type RawWindow = objc2::rc::Retained<objc2_app_kit::NSWindow>;
    } else {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        enum BorrowedWindowInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::Window),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
    }
}

/// Raw window handle.
#[derive(Clone, Copy)]
pub struct BorrowedWindow<'a>(BorrowedWindowInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedWindow<'a> {
    /// Create from Win32 `HWND`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `hwnd` is a valid handle for the lifetime
    /// `'a`.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(hwnd: windows_sys::Win32::Foundation::HWND) -> Self {
        Self(BorrowedWindowInner::Win32(hwnd, PhantomData))
    }

    /// Create from WinUI `Window`.
    #[cfg(feature = "winui")]
    pub fn winui(window: &'a winui3::Microsoft::UI::Xaml::Window) -> Self {
        Self(BorrowedWindowInner::WinUI(window))
    }

    /// Get Win32 `HWND`. Panic if the handle is not Win32.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::Foundation::HWND {
        match &self.0 {
            BorrowedWindowInner::Win32(hwnd, ..) => *hwnd,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get WinUI `Window`. Panic if the handle is not WinUI.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &'a winui3::Microsoft::UI::Xaml::Window {
        match &self.0 {
            BorrowedWindowInner::WinUI(window) => window,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get the raw window handle.
    pub fn handle(&self) -> windows_core::Result<windows_sys::Win32::Foundation::HWND> {
        match &self.0 {
            #[cfg(feature = "win32")]
            BorrowedWindowInner::Win32(hwnd, ..) => Ok(*hwnd),
            #[cfg(feature = "winui")]
            BorrowedWindowInner::WinUI(window) => unsafe {
                use windows_core::Interface;
                use winui3::IWindowNative;
                Ok(window.cast::<IWindowNative>()?.WindowHandle()?.0)
            },
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            BorrowedWindowInner::Dummy(a, ..) => match *a {},
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_os = "macos")))]
impl<'a> BorrowedWindow<'a> {
    /// Create from Qt `QWidget`.
    ///
    /// # Safety
    /// The caller must ensure that `widget` is a valid pointer for the lifetime
    /// `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(widget: *mut T) -> Self {
        Self(BorrowedWindowInner::Qt(widget.cast(), PhantomData))
    }

    /// Create from Gtk `Window`.
    #[cfg(feature = "gtk")]
    pub fn gtk(window: &'a gtk4::Window) -> Self {
        Self(BorrowedWindowInner::Gtk(window))
    }

    /// Get Qt `QWidget`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedWindowInner::Qt(w, ..) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `Window`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::Window {
        match &self.0 {
            BorrowedWindowInner::Gtk(w) => w,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[cfg(feature = "raw-window-handle")]
impl<'a> HasWindowHandle for BorrowedWindow<'a> {
    #[cfg(windows)]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.handle()
            .map(|hwnd| {
                use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
                unsafe {
                    WindowHandle::borrow_raw(RawWindowHandle::Win32(Win32WindowHandle::new(
                        std::num::NonZeroIsize::new(hwnd as _).expect("HWND is null"),
                    )))
                }
            })
            .map_err(|_| HandleError::NotSupported)
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

/// Trait to borrow the window handle.
pub trait AsWindow {
    /// Get the window handle.
    fn as_window(&self) -> BorrowedWindow<'_>;
}

impl AsWindow for BorrowedWindow<'_> {
    fn as_window(&self) -> BorrowedWindow<'_> {
        *self
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

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_window {
    ($t:ty, $inner:ident) => {
        impl $crate::AsWindow for $t {
            fn as_window(&self) -> $crate::BorrowedWindow<'_> {
                self.$inner.as_window()
            }
        }
    };
}
