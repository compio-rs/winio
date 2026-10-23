//! Handles for winio.
//!
//! # Window
//!
//! A window handle represents a window chrome, such as title bar and borders.
//! It interoperates with the underlying windowing system, such as Win32 or
//! WinUI on Windows, and AppKit on macOS.
//!
//! A window is usually both a window and a container, so it implements both
//! [`AsWindow`] and [`AsContainer`] traits.
//!
//! | Platform | Native Type      |
//! | -------- | ---------------- |
//! | Win32    | [`HWND`]         |
//! | WinUI    | [`Window`]       |
//! | Qt       | [`QWidget`]      |
//! | Gtk      | [`GtkWindow`]    |
//! | AppKit   | [`NSWindow`]     |
//! | UIKit    | [`UIWindow`]     |
//! | Android  | [`GameActivity`] |
//!
//! ## Platform specific
//! * Qt: The window is a [`QMainWindow`].
//!
//! # Container
//!
//! A container handle represents a container, such as a window or a scroll
//! view.
//!
//! | Platform | Native Type     |
//! | -------- | --------------- |
//! | Win32    | [`HWND`]        |
//! | WinUI    | [`Canvas`]      |
//! | Qt       | [`QWidget`]     |
//! | Gtk      | [`GtkFixed`]    |
//! | AppKit   | [`NSView`]      |
//! | UIKit    | [`UIView`]      |
//! | Android  | [`FrameLayout`] |
//!
//! ## Platform specific
//! * WinUI: The parent of all widgets must be a [`Canvas`], and the widget will
//!   be set to center alignment. The window of WinUI contains a [`Canvas`].
//! * Gtk: The parent of all widgets must be a [`GtkFixed`]. The window of Gtk
//!   contains a [`GtkScrolledWindow`], and the scrolled window contains a
//!   [`GtkFixed`].
//!
//! # Widget
//!
//! A widget handle represents a widget, such as a button or a text box.
//!
//! | Platform | Native Type          |
//! | -------- | -------------------- |
//! | Win32    | [`HWND`]             |
//! | WinUI    | [`FrameworkElement`] |
//! | Qt       | [`QWidget`]          |
//! | Gtk      | [`GtkWidget`]        |
//! | AppKit   | [`NSView`]           |
//! | UIKit    | [`UIView`]           |
//! | Android  | [`View`]             |
//!
//! # Menu Container
//!
//! A menu container handle represents a (sub)menu in a menu tree, such as a
//! top-level menu on a menu bar or a context menu. It holds the menu items of
//! that menu.
//!
//! | Platform | Native Type                                                    |
//! | -------- | -------------------------------------------------------------- |
//! | Win32    | [`HMENU`]                                                      |
//! | WinUI    | [`MenuBarItem`] <br> [`MenuFlyout`] <br> [`MenuFlyoutSubItem`] |
//! | Qt       | [`QMenu`]                                                      |
//! | Gtk      | [`GMenu`]                                                      |
//! | AppKit   | [`NSMenu`]                                                     |
//! | UIKit    | Key of a menu tree node                                        |
//! | Android  | [`Menu`]                    .                                  |
//!
//! ## Platform specific
//! * WinUI: A menu container is a [`MenuBarItem`], a [`MenuFlyout`], or a
//!   [`MenuFlyoutSubItem`], depending on its position in the menu tree.
//! * UIKit: The menu tree is not exposed as native objects, so the handle is a
//!   key of a node in the menu tree.
//!
//! # Menu Item
//!
//! A menu item handle represents an entry of a menu, such as a selectable item
//! or a separator.
//!
//! A menu item that opens a submenu is also a menu container, since it holds
//! the items of the submenu. It implements both [`AsMenuItem`] and
//! [`AsMenuContainer`] traits.
//!
//! | Platform | Native Type                |
//! | -------- | -------------------------- |
//! | Win32    | [`HMENU`] and a command ID |
//! | WinUI    | [`MenuFlyoutItemBase`]     |
//! | Qt       | [`QAction`]                |
//! | Gtk      | [`GMenuItem`]              |
//! | AppKit   | [`NSMenuItem`]             |
//! | UIKit    | Key of a menu tree node    |
//! | Android  | [`MenuItem`]               |
//!
//! ## Platform specific
//! * UIKit: The menu tree is not exposed as native objects, so the handle is a
//!   key of a node in the menu tree.
//!
//! # ToolBar
//!
//! A tool bar item handle represents an item of a toolbar, such as a button, a
//! check box, or a drop-down button.
//!
//! | Platform | Native Type               |
//! | -------- | ------------------------- |
//! | Win32    | [`HWND`] and a command ID |
//! | WinUI    | [`ICommandBarElement`]    |
//! | Qt       | [`QAction`]               |
//! | Gtk      | [`GtkWidget`]             |
//! | AppKit   | [`NSToolbarItem`]         |
//! | UIKit    | [`UIBarButtonItem`]       |
//! | Android  | [`MenuItem`]              |
//!
//! [`HWND`]: https://learn.microsoft.com/en-us/windows/apps/develop/ui/retrieve-hwnd
//! [`HMENU`]: https://learn.microsoft.com/en-us/windows/win32/menurc/using-menus
//! [`Window`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.window
//! [`Canvas`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.canvas
//! [`FrameworkElement`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.frameworkelement
//! [`MenuBarItem`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.menubaritem
//! [`MenuFlyout`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.menuflyout
//! [`MenuFlyoutSubItem`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.menuflyoutsubitem
//! [`MenuFlyoutItemBase`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.menuflyoutitembase
//! [`ICommandBarElement`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.icommandbarelement
//! [`QWidget`]: https://doc.qt.io/qt-6/qwidget.html
//! [`QMainWindow`]: https://doc.qt.io/qt-6/qmainwindow.html
//! [`QMenu`]: https://doc.qt.io/qt-6/qmenu.html
//! [`QAction`]: https://doc.qt.io/qt-6/qaction.html
//! [`GtkWindow`]: https://docs.gtk.org/gtk4/class.Window.html
//! [`GtkFixed`]: https://docs.gtk.org/gtk4/class.Fixed.html
//! [`GtkScrolledWindow`]: https://docs.gtk.org/gtk4/class.ScrolledWindow.html
//! [`GtkWidget`]: https://docs.gtk.org/gtk4/class.Widget.html
//! [`GMenu`]: https://docs.gtk.org/gio/class.Menu.html
//! [`GMenuItem`]: https://docs.gtk.org/gio/class.MenuItem.html
//! [`NSWindow`]: https://developer.apple.com/documentation/appkit/nswindow
//! [`NSView`]: https://developer.apple.com/documentation/appkit/nsview
//! [`NSMenu`]: https://developer.apple.com/documentation/appkit/nsmenu
//! [`NSMenuItem`]: https://developer.apple.com/documentation/appkit/nsmenuitem
//! [`NSToolbarItem`]: https://developer.apple.com/documentation/appkit/nstoolbaritem
//! [`UIWindow`]: https://developer.apple.com/documentation/uikit/uiwindow
//! [`UIView`]: https://developer.apple.com/documentation/uikit/uiview
//! [`UIBarButtonItem`]: https://developer.apple.com/documentation/uikit/uibarbuttonitem
//! [`GameActivity`]: https://developer.android.com/games/agdk/game-activity
//! [`FrameLayout`]: https://developer.android.com/reference/android/widget/FrameLayout
//! [`View`]: https://developer.android.com/reference/android/view/View
//! [`Menu`]: https://developer.android.com/reference/android/view/Menu
//! [`MenuItem`]: https://developer.android.com/reference/android/view/MenuItem

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod window;
pub use window::*;

mod widget;
pub use widget::*;

mod container;
pub use container::*;

mod menu;
pub use menu::*;

mod tool_bar;
pub use tool_bar::*;
