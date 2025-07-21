use std::{marker::PhantomData, ops::Deref};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        /// Raw widget handle.
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
    } else if #[cfg(target_os = "android")] {
        /// Android View or Widget
        pub type RawWidget = jni::objects::GlobalRef;
    } else {
        /// Raw widget handle.
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
#[cfg(not(any(windows, target_os = "macos", target_os = "android")))]
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

/// A borrowed widget handle.
#[derive(Clone)]
pub struct BorrowedWidget<'a> {
    handle: RawWidget,
    _p: PhantomData<&'a ()>,
}

impl BorrowedWidget<'_> {
    /// # Safety
    ///
    /// The widget must remain valid for the duration of the returned
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

/// Trait to exact the raw widget handle.
pub trait AsRawWidget {
    /// Get the raw widget handle.
    fn as_raw_widget(&self) -> RawWidget;

    /// Iterate all raw widget handles.
    ///
    /// This is useful for widgets that are implemented by multiple raw widgets.
    fn iter_raw_widgets(&self) -> impl Iterator<Item = RawWidget> {
        std::iter::once(self.as_raw_widget())
    }
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

    fn iter_raw_widgets(&self) -> impl Iterator<Item = RawWidget> {
        (**self).iter_raw_widgets()
    }
}

/// Trait to borrow the widget handle.
pub trait AsWidget {
    /// Get the widget handle.
    fn as_widget(&self) -> BorrowedWidget<'_>;

    /// Iterate all widget handles. See [`AsRawWidget::iter_raw_widgets`].
    fn iter_widgets(&self) -> impl Iterator<Item = BorrowedWidget<'_>> {
        std::iter::once(self.as_widget())
    }
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

    #[inline]
    fn iter_widgets(&self) -> impl Iterator<Item = BorrowedWidget<'_>> {
        T::iter_widgets(self)
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
        impl $crate::AsRawWidget for $t {
            fn as_raw_widget(&self) -> $crate::RawWidget {
                self.$inner.as_raw_widget()
            }
        }
        impl $crate::AsWidget for $t {
            fn as_widget(&self) -> $crate::BorrowedWidget<'_> {
                unsafe {
                    $crate::BorrowedWidget::borrow_raw($crate::AsRawWidget::as_raw_widget(self))
                }
            }
        }
    };
    ($t:ty) => {
        impl $crate::AsWidget for $t {
            fn as_widget(&self) -> $crate::BorrowedWidget<'_> {
                unsafe {
                    $crate::BorrowedWidget::borrow_raw($crate::AsRawWidget::as_raw_widget(self))
                }
            }

            fn iter_widgets(&self) -> impl core::iter::Iterator<Item = $crate::BorrowedWidget<'_>> {
                unsafe {
                    $crate::AsRawWidget::iter_raw_widgets(self)
                        .map(|w| $crate::BorrowedWidget::borrow_raw(w))
                }
            }
        }
    };
}
