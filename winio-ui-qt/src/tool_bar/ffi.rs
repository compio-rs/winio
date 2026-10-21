#[cxx::bridge]
mod bindings {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/tool_bar/ffi.hpp");

        type QWidget = crate::widgets::QWidget;
        type QToolBar;
        type QAction = crate::menu::QAction;
        type QMenu = crate::menu::QMenu;

        unsafe fn new_tool_bar(window: *mut QWidget) -> Result<UniquePtr<QToolBar>>;

        unsafe fn tool_bar_exists(window: *mut QWidget) -> bool;

        unsafe fn tool_bar_add_action(toolbar: *mut QToolBar) -> Result<*mut QAction>;

        unsafe fn tool_bar_add_separator(toolbar: *mut QToolBar) -> Result<*mut QAction>;

        unsafe fn tool_bar_add_menu_action(toolbar: *mut QToolBar) -> Result<*mut QAction>;

        unsafe fn tool_bar_action_menu(action: *mut QAction) -> Result<*mut QMenu>;
    }
}

pub(crate) use bindings::*;
