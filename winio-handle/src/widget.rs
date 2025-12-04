cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        enum BorrowedWidgetInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::Foundation::HWND, PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::FrameworkElement),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
    } else if #[cfg(target_os = "macos")] {
        /// [`NSView`].
        ///
        /// [`NSView`]: objc2_app_kit::NSView
        pub type RawWidget = objc2::rc::Retained<objc2_app_kit::NSView>;
    } else {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        enum BorrowedWidgetInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::Widget),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::convert::Infallible, PhantomData<&'a ()>),
        }
    }
}

/// Raw widget handle.
#[derive(Clone, Copy)]
pub struct BorrowedWidget<'a>(BorrowedWidgetInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedWidget<'a> {
    /// Create from Win32 `HWND`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `hwnd` is a valid handle for the lifetime
    /// `'a`.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(hwnd: windows_sys::Win32::Foundation::HWND) -> Self {
        Self(BorrowedWidgetInner::Win32(hwnd, PhantomData))
    }

    /// Create from WinUI `FrameworkElement`.
    #[cfg(feature = "winui")]
    pub fn winui(element: &'a winui3::Microsoft::UI::Xaml::FrameworkElement) -> Self {
        Self(BorrowedWidgetInner::WinUI(element))
    }

    /// Get Win32 `HWND`.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::Foundation::HWND {
        match &self.0 {
            BorrowedWidgetInner::Win32(hwnd, ..) => *hwnd,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get WinUI `FrameworkElement`.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &winui3::Microsoft::UI::Xaml::FrameworkElement {
        match &self.0 {
            BorrowedWidgetInner::WinUI(e) => e,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_os = "macos")))]
impl<'a> BorrowedWidget<'a> {
    /// Create from Qt `QWidget`.
    ///
    /// # Safety
    /// The caller must ensure that `widget` is a valid pointer for the lifetime
    /// `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(widget: *mut T) -> Self {
        Self(BorrowedWidgetInner::Qt(widget.cast(), PhantomData))
    }

    /// Create from Gtk `Widget`.
    #[cfg(feature = "gtk")]
    pub fn gtk(widget: &'a gtk4::Widget) -> Self {
        Self(BorrowedWidgetInner::Gtk(widget))
    }

    /// Get Qt `QWidget`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedWidgetInner::Qt(w, ..) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `Widget`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::Widget {
        match &self.0 {
            BorrowedWidgetInner::Gtk(w) => w,
            _ => panic!("unsupported handle type"),
        }
    }
}

/// Trait to borrow the widget handle.
pub trait AsWidget {
    /// Get the widget handle.
    fn as_widget(&self) -> BorrowedWidget<'_>;
}

impl AsWidget for BorrowedWidget<'_> {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        *self
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

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_widget {
    ($t:ty, $inner:ident) => {
        impl $crate::AsWidget for $t {
            fn as_widget(&self) -> $crate::BorrowedWidget<'_> {
                self.$inner.as_widget()
            }
        }
    };
}
