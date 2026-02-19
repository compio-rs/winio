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
//! | Platform | Native Type   |
//! | -------- | ------------- |
//! | Win32    | [`HWND`]      |
//! | WinUI    | [`Window`]    |
//! | Qt       | [`QWidget`]   |
//! | Gtk      | [`GtkWindow`] |
//! | AppKit   | [`NSWindow`]  |
//!
//! ## Platform specific
//! * Qt: The window is a [`QMainWindow`].
//!
//! # Container
//!
//! A container handle represents a container, such as a window or a scroll
//! view.
//!
//! | Platform | Native Type   |
//! | -------- | ------------- |
//! | Win32    | [`HWND`]      |
//! | WinUI    | [`Canvas`]    |
//! | Qt       | [`QWidget`]   |
//! | Gtk      | [`GtkFixed`]  |
//! | AppKit   | [`NSView`]    |
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
//!
//! [`HWND`]: https://learn.microsoft.com/en-us/windows/apps/develop/ui/retrieve-hwnd
//! [`Window`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.window
//! [`Canvas`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.canvas
//! [`FrameworkElement`]: https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.frameworkelement
//! [`QWidget`]: https://doc.qt.io/qt-6/qwidget.html
//! [`QMainWindow`]: https://doc.qt.io/qt-6/qmainwindow.html
//! [`GtkWindow`]: https://docs.gtk.org/gtk4/class.Window.html
//! [`GtkFixed`]: https://docs.gtk.org/gtk4/class.Fixed.html
//! [`GtkScrolledWindow`]: https://docs.gtk.org/gtk4/class.ScrolledWindow.html
//! [`GtkWidget`]: https://docs.gtk.org/gtk4/class.Widget.html
//! [`NSWindow`]: https://developer.apple.com/documentation/appkit/nswindow
//! [`NSView`]: https://developer.apple.com/documentation/appkit/nsview

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod window;
pub use window::*;

mod widget;
pub use widget::*;

mod container;
pub use container::*;
