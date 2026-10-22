cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[derive(Clone, Copy)]
        enum BorrowedToolBarItemInner<'a> {
            #[cfg(feature = "win32")]
            Win32(
                windows_sys::Win32::Foundation::HWND,
                u32,
                std::marker::PhantomData<&'a ()>,
            ),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::Controls::ICommandBarElement),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    } else {
        #[derive(Clone, Copy)]
        enum BorrowedToolBarItemInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::Widget),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    }
}

/// Raw tool bar item handle.
#[derive(Clone, Copy)]
pub struct BorrowedToolBarItem<'a>(BorrowedToolBarItemInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedToolBarItem<'a> {
    /// Create from Win32 toolbar `HWND` and command ID.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that `hwnd` is a valid toolbar handle for the
    ///   lifetime `'a`.
    /// * `hwnd` must not be null.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(hwnd: windows_sys::Win32::Foundation::HWND, id: u32) -> Self {
        Self(BorrowedToolBarItemInner::Win32(
            hwnd,
            id,
            std::marker::PhantomData,
        ))
    }

    /// Create from WinUI `ICommandBarElement`.
    #[cfg(feature = "winui")]
    pub fn winui(element: &'a winui3::Microsoft::UI::Xaml::Controls::ICommandBarElement) -> Self {
        Self(BorrowedToolBarItemInner::WinUI(element))
    }

    /// Get the Win32 toolbar `HWND` and command ID. Panic if the handle is not
    /// Win32.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> (windows_sys::Win32::Foundation::HWND, u32) {
        match self.0 {
            BorrowedToolBarItemInner::Win32(hwnd, id, ..) => (hwnd, id),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get the WinUI `ICommandBarElement`. Panic if the handle is not WinUI.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &'a winui3::Microsoft::UI::Xaml::Controls::ICommandBarElement {
        match self.0 {
            BorrowedToolBarItemInner::WinUI(element) => element,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
impl<'a> BorrowedToolBarItem<'a> {
    /// Create from Qt `QAction`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `action` is a valid pointer for the lifetime
    /// `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(action: *mut T) -> Self {
        Self(BorrowedToolBarItemInner::Qt(
            action.cast(),
            std::marker::PhantomData,
        ))
    }

    /// Get Qt `QAction`. Panic if the handle is not Qt.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedToolBarItemInner::Qt(a, ..) => (*a).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Create from a Gtk widget.
    #[cfg(feature = "gtk")]
    pub fn gtk(widget: &'a gtk4::Widget) -> Self {
        Self(BorrowedToolBarItemInner::Gtk(widget))
    }

    /// Get the Gtk widget. Panic if the handle is not Gtk.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::Widget {
        match self.0 {
            BorrowedToolBarItemInner::Gtk(widget) => widget,
            _ => panic!("unsupported handle type"),
        }
    }
}

/// Trait to borrow the tool bar item handle.
pub trait AsToolBarItem {
    /// Get the tool bar item handle.
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_>;
}

impl AsToolBarItem for BorrowedToolBarItem<'_> {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        *self
    }
}

impl<T: AsToolBarItem + ?Sized> AsToolBarItem for &T {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        (*self).as_tool_bar_item()
    }
}

impl<'a, T: AsToolBarItem + ?Sized> From<&'a T> for BorrowedToolBarItem<'a> {
    fn from(value: &'a T) -> Self {
        value.as_tool_bar_item()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_tool_bar_item {
    ($t:ty, $inner:ident) => {
        impl $crate::AsToolBarItem for $t {
            fn as_tool_bar_item(&self) -> $crate::BorrowedToolBarItem<'_> {
                self.$inner.as_tool_bar_item()
            }
        }
    };
}
