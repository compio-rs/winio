cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[derive(Clone, Copy)]
        enum BorrowedMenuContainerInner<'a> {
            #[cfg(feature = "win32")]
            Win32(windows_sys::Win32::UI::WindowsAndMessaging::HMENU, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "winui")]
            WinUIMenuBar(&'a winui3::Microsoft::UI::Xaml::Controls::MenuBarItem),
            #[cfg(feature = "winui")]
            WinUIMenuItem(&'a winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutSubItem),
            #[cfg(not(any(feature = "win32", feature = "winui")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    } else if #[cfg(target_os = "macos")] {
        use objc2::rc::Retained;

        type BorrowedMenuContainerInner<'a> = &'a Retained<objc2_app_kit::NSMenu>;
    } else {
        #[derive(Clone, Copy)]
        enum BorrowedMenuContainerInner<'a> {
            #[cfg(feature = "qt")]
            Qt(*mut core::ffi::c_void, std::marker::PhantomData<&'a ()>),
            #[cfg(feature = "gtk")]
            Gtk(&'a gtk4::gio::Menu),
            #[cfg(not(any(feature = "qt", feature = "gtk")))]
            Dummy(std::marker::PhantomData<&'a ()>),
        }
    }
}

/// Raw menu container handle.
#[derive(Clone, Copy)]
pub struct BorrowedMenuContainer<'a>(BorrowedMenuContainerInner<'a>);

#[allow(unreachable_patterns)]
#[cfg(windows)]
impl<'a> BorrowedMenuContainer<'a> {
    /// Create from Win32 `HMENU`.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that `hmenu` is a valid handle for the lifetime
    ///   `'a`.
    /// * `hmenu` must not be null.
    #[cfg(feature = "win32")]
    pub unsafe fn win32(hmenu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU) -> Self {
        Self(BorrowedMenuContainerInner::Win32(
            hmenu,
            std::marker::PhantomData,
        ))
    }

    /// Create from WinUI `MenuBarItem`.
    #[cfg(feature = "winui")]
    pub fn winui_menu_bar(
        menu_item: &'a winui3::Microsoft::UI::Xaml::Controls::MenuBarItem,
    ) -> Self {
        Self(BorrowedMenuContainerInner::WinUIMenuBar(menu_item))
    }

    /// Create from WinUI `MenuItem`.
    #[cfg(feature = "winui")]
    pub fn winui_menu_item(
        menu_item: &'a winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutSubItem,
    ) -> Self {
        Self(BorrowedMenuContainerInner::WinUIMenuItem(menu_item))
    }

    /// Get the Win32 `HMENU` handle. Panic if the handle is not Win32.
    #[cfg(feature = "win32")]
    pub fn as_win32(&self) -> windows_sys::Win32::UI::WindowsAndMessaging::HMENU {
        match self.0 {
            BorrowedMenuContainerInner::Win32(hmenu, _) => hmenu,
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get the WinUI `MenuBarItem`. Panic if the handle is not WinUI.
    #[cfg(feature = "winui")]
    pub fn as_winui_menu_bar(
        &self,
    ) -> Option<&'a winui3::Microsoft::UI::Xaml::Controls::MenuBarItem> {
        match self.0 {
            BorrowedMenuContainerInner::WinUIMenuBar(menu_item) => Some(menu_item),
            _ => None,
        }
    }

    /// Get the WinUI `MenuItem`. Panic if the handle is not WinUI.
    #[cfg(feature = "winui")]
    pub fn as_winui_menu_item(
        &self,
    ) -> Option<&'a winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutSubItem> {
        match self.0 {
            BorrowedMenuContainerInner::WinUIMenuItem(menu_item) => Some(menu_item),
            _ => None,
        }
    }
}

#[cfg(target_os = "macos")]
impl<'a> BorrowedMenuContainer<'a> {
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
impl<'a> BorrowedMenuContainer<'a> {
    /// Create from Qt `QMenu`.
    ///
    /// # Safety
    /// The caller must ensure that `menu` is a valid pointer for the lifetime
    /// `'a`.
    #[cfg(feature = "qt")]
    pub unsafe fn qt<T>(menu: *mut T) -> Self {
        Self(BorrowedMenuContainerInner::Qt(
            menu.cast(),
            std::marker::PhantomData,
        ))
    }

    /// Create from Gtk `gio::Menu`.
    #[cfg(feature = "gtk")]
    pub fn gtk(menu: &'a gtk4::gio::Menu) -> Self {
        Self(BorrowedMenuContainerInner::Gtk(menu))
    }

    /// Get Qt `QMenu`.
    #[cfg(feature = "qt")]
    pub fn as_qt<T>(&self) -> *mut T {
        match &self.0 {
            BorrowedMenuContainerInner::Qt(m, ..) => (*m).cast(),
            _ => panic!("unsupported handle type"),
        }
    }

    /// Get Gtk `gio::Menu`.
    #[cfg(feature = "gtk")]
    pub fn to_gtk(&self) -> &'a gtk4::gio::Menu {
        match &self.0 {
            BorrowedMenuContainerInner::Gtk(m) => m,
            _ => panic!("unsupported handle type"),
        }
    }
}

/// Trait to borrow the menu container handle.
pub trait AsMenuContainer {
    /// Get the menu container handle.
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_>;
}

impl AsMenuContainer for BorrowedMenuContainer<'_> {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        *self
    }
}

impl<T: AsMenuContainer + ?Sized> AsMenuContainer for &T {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        (*self).as_menu_container()
    }
}

impl<'a, T: AsMenuContainer + ?Sized> From<&'a T> for BorrowedMenuContainer<'a> {
    fn from(value: &'a T) -> Self {
        value.as_menu_container()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_menu_container {
    ($t:ty, $inner:ident) => {
        impl $crate::AsMenuContainer for $t {
            fn as_menu_container(&self) -> $crate::BorrowedMenuContainer<'_> {
                self.$inner.as_menu_container()
            }
        }
    };
}
