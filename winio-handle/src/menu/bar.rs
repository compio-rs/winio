cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[derive(Clone, Copy)]
        enum BorrowedMenuBarInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::UI::WindowsAndMessaging::HMENU, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::Controls::MenuBar),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    } else if #[cfg(target_os = "macos")] {
        use objc2::rc::Retained;

        type BorrowedMenuBarInner<'a> = &'a Retained<objc2_app_kit::NSMenu>;
    } else {
        #[derive(Clone, Copy)]
        enum BorrowedMenuBarInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::gio::Menu),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    }
}

/// Raw menu bar handle.
#[derive(Clone, Copy)]
pub struct BorrowedMenuBar<'a>(BorrowedMenuBarInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedMenuBar<'a> {
    /// Create from Win32 `HMENU`.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that `hmenu` is a valid handle for the lifetime
    ///   `'a`.
    /// * `hmenu` must not be null.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(hmenu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU) -> Self {
        Self(BorrowedMenuBarInner::Win32(hmenu, std::marker::PhantomData))
    }

    /// Create from WinUI `MenuBar`.
    #[cfg(feature = "winui")]
    pub fn winui(menu_bar: &'a winui3::Microsoft::UI::Xaml::Controls::MenuBar) -> Self {
        Self(BorrowedMenuBarInner::WinUI(menu_bar))
    }

    /// Get the Win32 `HMENU` handle. Panic if the handle is not Win32.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::UI::WindowsAndMessaging::HMENU {
        match self.0 {
            BorrowedMenuBarInner::Win32(hmenu, _) => hmenu,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get the WinUI `MenuBar`. Panic if the handle is not WinUI.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &'a winui3::Microsoft::UI::Xaml::Controls::MenuBar {
        match self.0 {
            BorrowedMenuBarInner::WinUI(menu_bar) => menu_bar,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[cfg(target_os = "macos")]
impl<'a> BorrowedMenuBar<'a> {
    /// Create from `NSMenu`.
    pub fn app_kit(menu: &'a Retained<objc2_app_kit::NSMenu>) -> Self {
        Self(menu)
    }

    /// Get `NSMenu`.
    pub fn as_app_kit(&self) -> &'a Retained<objc2_app_kit::NSMenu> {
        self.0
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
impl<'a> BorrowedMenuBar<'a> {
    /// Create from Qt `QMenuBar`.
    ///
    /// # Safety
    /// The caller must ensure that `menu_bar` is a valid pointer for the
    /// lifetime `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(menu_bar: *mut T) -> Self {
        Self(BorrowedMenuBarInner::Qt(
            menu_bar.cast(),
            std::marker::PhantomData,
        ))
    }

    /// Create from Gtk `PopoverMenuBar`.
    #[cfg(feature = "gtk")]
    pub fn gtk(menu_bar: &'a gtk4::gio::Menu) -> Self {
        Self(BorrowedMenuBarInner::Gtk(menu_bar))
    }

    /// Get Qt `QMenuBar`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedMenuBarInner::Qt(w, ..) => (*w).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `PopoverMenuBar`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::gio::Menu {
        match &self.0 {
            BorrowedMenuBarInner::Gtk(w) => w,
            _ => panic!("unsupported handle type"),
        }
    }
}

/// Trait to borrow the menu bar handle.
pub trait AsMenuBar {
    /// Get the menu bar handle.
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_>;
}

impl AsMenuBar for BorrowedMenuBar<'_> {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        *self
    }
}

impl<T: AsMenuBar + ?Sized> AsMenuBar for &T {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        (*self).as_menu_bar()
    }
}

impl<'a, T: AsMenuBar + ?Sized> From<&'a T> for BorrowedMenuBar<'a> {
    fn from(value: &'a T) -> Self {
        value.as_menu_bar()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_menu_bar {
    ($t:ty, $inner:ident) => {
        impl $crate::AsMenuBar for $t {
            fn as_menu_bar(&self) -> $crate::BorrowedMenuBar<'_> {
                self.$inner.as_menu_bar()
            }
        }
    };
}
