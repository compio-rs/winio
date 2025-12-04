cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        enum BorrowedContainerInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND, PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::Controls::Canvas),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
    } else if #[cfg(target_os = "macos")] {
        use objc2::rc::Retained;

        type BorrowedContainerInner<'a> = &'a Retained<objc2_app_kit::NSView>;
    } else {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        enum BorrowedContainerInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::Fixed),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
    }
}

/// Raw container handle.
#[derive(Clone, Copy)]
pub struct BorrowedContainer<'a>(BorrowedContainerInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedContainer<'a> {
    /// Create from Win32 `HWND`.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that `hwnd` is a valid handle for the lifetime
    ///   `'a`.
    /// * `hwnd` must not be null.
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

#[cfg(target_os = "macos")]
impl<'a> BorrowedContainer<'a> {
    /// Create from `NSView`.
    pub fn app_kit(view: &'a Retained<objc2_app_kit::NSView>) -> Self {
        Self(view)
    }

    /// Get `NSView`.
    pub fn as_app_kit(&self) -> &'a Retained<objc2_app_kit::NSView> {
        self.0
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_os = "macos")))]
impl<'a> BorrowedContainer<'a> {
    /// Create from Qt `QWidget`.
    ///
    /// # Safety
    /// The caller must ensure that `widget` is a valid pointer for the lifetime
    /// `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(widget: *mut T) -> Self {
        Self(BorrowedContainerInner::Qt(widget.cast(), PhantomData))
    }

    /// Create from Gtk `Fixed`.
    #[cfg(feature = "gtk")]
    pub fn gtk(fixed: &'a gtk4::Fixed) -> Self {
        Self(BorrowedContainerInner::Gtk(fixed))
    }

    /// Get Qt `QWidget`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedContainerInner::Qt(w, ..) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `Fixed`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::Fixed {
        match &self.0 {
            BorrowedContainerInner::Gtk(w) => w,
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
