use std::marker::PhantomData;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[derive(Clone, Copy)]
        enum BorrowedContainerInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND, PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::Controls::Canvas),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
        /// Raw container handle.
        #[derive(Clone, Copy)]
        #[non_exhaustive]
        pub struct BorrowedContainer<'a>(BorrowedContainerInner<'a>);
    } else if #[cfg(target_os = "macos")] {
        /// [`NSView`].
        ///
        /// [`NSView`]: objc2_app_kit::NSView
        pub type RawContainer = objc2::rc::Retained<objc2_app_kit::NSView>;
    } else {
        /// Raw container handle.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum RawContainer {
            /// Pointer to `QWidget`.
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void),
            /// GTK [`Fixed`].
            ///
            /// [`Fixed`]: gtk4::Fixed
            #[cfg(feature = "gtk")]
            Gtk(gtk4::Fixed),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedContainer<'a> {
    /// Create from Win32 `HWND`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `hwnd` is a valid handle for the lifetime
    /// `'a`.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(hwnd: windows_sys::Win32::Foundation::HWND) -> Self {
        Self(BorrowedContainerInner::Win32(hwnd, PhantomData))
    }

    /// Create from WinUI `Canvas`.
    #[cfg(feature = "winui")]
    pub fn winui(container: &'a winui3::Microsoft::UI::Xaml::Controls::Canvas) -> Self {
        Self(BorrowedContainerInner::WinUI(container))
    }

    /// Get Win32 `HWND`.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::Foundation::HWND {
        match &self.0 {
            BorrowedContainerInner::Win32(hwnd, ..) => *hwnd,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get WinUI `Canvas`.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &winui3::Microsoft::UI::Xaml::Controls::Canvas {
        match &self.0 {
            BorrowedContainerInner::WinUI(container) => container,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_os = "macos")))]
impl RawContainer {
    /// Get Qt `QWidget`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match self {
            Self::Qt(w) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `Fixed`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> gtk4::Fixed {
        match self {
            Self::Gtk(w) => w.clone(),
            _ => panic!("unsupported handle type"),
        }
    }
}

/// Trait to borrow the container handle.
pub trait AsContainer {
    /// Get the container handle.
    fn as_container(&self) -> BorrowedContainer<'_>;
}

impl AsContainer for BorrowedContainer<'_> {
    fn as_container(&self) -> BorrowedContainer<'_> {
        *self
    }
}

impl<T: AsContainer + ?Sized> AsContainer for &T {
    #[inline]
    fn as_container(&self) -> BorrowedContainer<'_> {
        T::as_container(self)
    }
}

impl<'a, T: AsContainer + ?Sized> From<&'a T> for BorrowedContainer<'a> {
    fn from(value: &'a T) -> Self {
        value.as_container()
    }
}

#[doc(hidden)]
pub struct MaybeBorrowedContainer<'a>(pub Option<BorrowedContainer<'a>>);

impl<'a, T: Into<BorrowedContainer<'a>>> From<T> for MaybeBorrowedContainer<'a> {
    fn from(value: T) -> Self {
        Self(Some(value.into()))
    }
}

impl<'a, T: Into<BorrowedContainer<'a>>> From<Option<T>> for MaybeBorrowedContainer<'a> {
    fn from(value: Option<T>) -> Self {
        Self(value.map(|v| v.into()))
    }
}

impl From<()> for MaybeBorrowedContainer<'_> {
    fn from(_: ()) -> Self {
        Self(None)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_container {
    ($t:ty, $inner:ident) => {
        impl $crate::AsContainer for $t {
            fn as_container(&self) -> $crate::BorrowedContainer<'_> {
                self.$inner.as_container()
            }
        }
    };
}
