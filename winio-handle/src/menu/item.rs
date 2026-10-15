cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[derive(Clone, Copy)]
        enum BorrowedMenuItemInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::UI::WindowsAndMessaging::HMENU, u32, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUI(&'a winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutItemBase),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    } else if #[cfg(target_os = "macos")] {
        use objc2::rc::Retained;

        type BorrowedMenuItemInner<'a> = &'a Retained<objc2_app_kit::NSMenuItem>;
    } else if #[cfg(target_os = "ios")] {
        #[derive(Clone, Copy)]
        struct BorrowedMenuItemInner<'a>(usize, std::marker::PhantomData<&'a ()>);
    } else {
        #[derive(Clone, Copy)]
        enum BorrowedMenuItemInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::gio::MenuItem),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    }
}

/// Raw menu item handle.
#[derive(Clone, Copy)]
pub struct BorrowedMenuItem<'a>(BorrowedMenuItemInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedMenuItem<'a> {
    /// Create from Win32 `HMENU`.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that `hmenu` is a valid handle for the lifetime
    ///   `'a`.
    /// * `hmenu` must not be null.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(
        hmenu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU,
        id: u32,
    ) -> Self {
        Self(BorrowedMenuItemInner::Win32(
            hmenu,
            id,
            std::marker::PhantomData,
        ))
    }

    /// Create from WinUI `MenuFlyoutItemBase`.
    #[cfg(feature = "winui")]
    pub fn winui(menu_item: &'a winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutItemBase) -> Self {
        Self(BorrowedMenuItemInner::WinUI(menu_item))
    }

    /// Get the Win32 `HMENU` handle. Panic if the handle is not Win32.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> (windows_sys::Win32::UI::WindowsAndMessaging::HMENU, u32) {
        match self.0 {
            BorrowedMenuItemInner::Win32(hmenu, id, ..) => (hmenu, id),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get the WinUI `MenuFlyoutItemBase`. Panic if the handle is not WinUI.
    #[cfg(feature = "winui")]
    pub fn as_winui(&self) -> &'a winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutItemBase {
        match self.0 {
            BorrowedMenuItemInner::WinUI(menu_item) => menu_item,
            _ => panic!("unsupported handle type"),
        }
    }
}

#[cfg(target_os = "macos")]
impl<'a> BorrowedMenuItem<'a> {
    /// Create from `NSMenuItem`.
    pub fn app_kit(menu_item: &'a Retained<objc2_app_kit::NSMenuItem>) -> Self {
        Self(menu_item)
    }

    /// Get `NSMenuItem`.
    pub fn as_app_kit(&self) -> &'a Retained<objc2_app_kit::NSMenuItem> {
        self.0
    }
}

#[cfg(target_os = "ios")]
impl<'a> BorrowedMenuItem<'a> {
    /// Create from the key of a node of the UIKit menu tree.
    pub fn ui_kit(key: usize) -> Self {
        Self(BorrowedMenuItemInner(key, std::marker::PhantomData))
    }

    /// Get the key of the node of the UIKit menu tree.
    pub fn as_ui_kit(&self) -> usize {
        self.0.0
    }
}

#[allow(unreachable_patterns)]
#[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
impl<'a> BorrowedMenuItem<'a> {
    /// Create from Qt `QAction`.
    ///
    /// # Safety
    /// The caller must ensure that `action` is a valid pointer for the lifetime
    /// `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(action: *mut T) -> Self {
        Self(BorrowedMenuItemInner::Qt(
            action.cast(),
            std::marker::PhantomData,
        ))
    }

    /// Create from Gtk `gio::MenuItem`.
    #[cfg(feature = "gtk")]
    pub fn gtk(menu_item: &'a gtk4::gio::MenuItem) -> Self {
        Self(BorrowedMenuItemInner::Gtk(menu_item))
    }

    /// Get Qt `QAction`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedMenuItemInner::Qt(a, ..) => (*a).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `gio::MenuItem`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::gio::MenuItem {
        match &self.0 {
            BorrowedMenuItemInner::Gtk(w) => w,
            _ => panic!("unsupported handle type"),
        }
    }
}

/// Trait to borrow the menu item handle.
pub trait AsMenuItem {
    /// Get the menu item handle.
    fn as_menu_item(&self) -> BorrowedMenuItem<'_>;
}

impl AsMenuItem for BorrowedMenuItem<'_> {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        *self
    }
}

impl<T: AsMenuItem + ?Sized> AsMenuItem for &T {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        (*self).as_menu_item()
    }
}

impl<'a, T: AsMenuItem + ?Sized> From<&'a T> for BorrowedMenuItem<'a> {
    fn from(value: &'a T) -> Self {
        value.as_menu_item()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_menu_item {
    ($t:ty, $inner:ident) => {
        impl $crate::AsMenuItem for $t {
            fn as_menu_item(&self) -> $crate::BorrowedMenuItem<'_> {
                self.$inner.as_menu_item()
            }
        }
    };
}
