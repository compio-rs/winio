use std::{marker::PhantomData, ops::Deref};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        /// Raw container handle.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum RawContainer {
            /// Win32 `HWND`.
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND),
            /// WinUI `Canvas`.
            #[cfg(feature = "winui")]
            WinUI(winui3::Microsoft::UI::Xaml::Controls::Canvas),
        }
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
            #[cfg(all(not(any(windows, target_os = "macos")), feature = "qt"))]
            Qt(*mut core::ffi::c_void),
            /// GTK [`Fixed`].
            ///
            /// [`Fixed`]: gtk4::Fixed
            #[cfg(all(not(any(windows, target_os = "macos")), feature = "gtk"))]
            Gtk(gtk4::Fixed),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl RawContainer {
    /// Get Win32 `HWND`.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::Foundation::HWND {
        match self {
            Self::Win32(hwnd) => *hwnd,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get WinUI `Canvas`.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &winui3::Microsoft::UI::Xaml::Controls::Canvas {
        match self {
            Self::WinUI(container) => container,
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

/// A borrowed container handle.
#[derive(Clone)]
pub struct BorrowedContainer<'a> {
    handle: RawContainer,
    _p: PhantomData<&'a ()>,
}

impl BorrowedContainer<'_> {
    /// # Safety
    ///
    /// The container must remain valid for the duration of the returned
    /// [`BorrowedContainer`].
    pub unsafe fn borrow_raw(handle: RawContainer) -> Self {
        Self {
            handle,
            _p: PhantomData,
        }
    }
}

impl Deref for BorrowedContainer<'_> {
    type Target = RawContainer;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Trait to exact the raw container handle.
pub trait AsRawContainer {
    /// Get the raw container handle.
    fn as_raw_container(&self) -> RawContainer;
}

impl AsRawContainer for RawContainer {
    #[allow(clippy::clone_on_copy)]
    fn as_raw_container(&self) -> RawContainer {
        self.clone()
    }
}

impl AsRawContainer for BorrowedContainer<'_> {
    #[allow(clippy::clone_on_copy)]
    fn as_raw_container(&self) -> RawContainer {
        self.handle.clone()
    }
}

impl<T: AsRawContainer> AsRawContainer for &'_ T {
    fn as_raw_container(&self) -> RawContainer {
        (**self).as_raw_container()
    }
}

/// Trait to borrow the container handle.
pub trait AsContainer {
    /// Get the container handle.
    fn as_container(&self) -> BorrowedContainer<'_>;
}

impl AsContainer for BorrowedContainer<'_> {
    fn as_container(&self) -> BorrowedContainer<'_> {
        self.clone()
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
        impl $crate::AsRawContainer for $t {
            fn as_raw_container(&self) -> $crate::RawContainer {
                self.$inner.as_raw_container()
            }
        }
        $crate::impl_as_container!($t);
    };
    ($t:ty) => {
        impl $crate::AsContainer for $t {
            fn as_container(&self) -> $crate::BorrowedContainer<'_> {
                unsafe {
                    $crate::BorrowedContainer::borrow_raw($crate::AsRawContainer::as_raw_container(
                        self,
                    ))
                }
            }
        }
    };
}
