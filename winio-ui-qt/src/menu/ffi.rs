#[cxx::bridge]
mod bindings {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/menu/ffi.hpp");

        type QWidget = crate::widgets::QWidget;
        type QString = crate::common::QString;
        type QMenuBar;
        type QMenu;
        type QAction;
        type QActionGroup;

        unsafe fn new_menu_bar(window: *mut QWidget) -> Result<*mut QMenuBar>;

        fn isVisible(self: &QMenuBar) -> Result<bool>;
        fn setVisible(self: Pin<&mut QMenuBar>, visible: bool) -> Result<()>;

        fn menuAction(self: &QMenu) -> Result<*mut QAction>;
        fn addSeparator(self: Pin<&mut QMenu>) -> Result<*mut QAction>;

        fn setCheckable(self: Pin<&mut QAction>, checkable: bool) -> Result<()>;
        fn text(self: &QAction) -> Result<QString>;
        fn setText(self: Pin<&mut QAction>, text: &QString) -> Result<()>;
        fn isEnabled(self: &QAction) -> Result<bool>;
        fn setEnabled(self: Pin<&mut QAction>, enabled: bool) -> Result<()>;
        fn isChecked(self: &QAction) -> Result<bool>;
        fn setChecked(self: Pin<&mut QAction>, checked: bool) -> Result<()>;

        unsafe fn menu_bar_add_menu(menu_bar: *mut QMenuBar) -> Result<*mut QMenu>;
        unsafe fn menu_add_menu(menu: *mut QMenu) -> Result<*mut QMenu>;
        unsafe fn menu_add_action(menu: *mut QMenu) -> Result<*mut QAction>;

        unsafe fn new_action_group(parent: *mut QMenu) -> Result<UniquePtr<QActionGroup>>;
        unsafe fn new_action(parent: *mut QMenu) -> Result<*mut QAction>;

        fn setExclusive(self: Pin<&mut QActionGroup>, exclusive: bool) -> Result<()>;
        unsafe fn addAction(
            self: Pin<&mut QActionGroup>,
            action: *mut QAction,
        ) -> Result<*mut QAction>;

        unsafe fn action_connect_triggered(
            action: Pin<&mut QAction>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;
    }
}

pub(crate) use bindings::*;
