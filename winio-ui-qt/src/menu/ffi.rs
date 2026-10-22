use std::mem::MaybeUninit;

use cxx::{ExternType, type_id};

#[repr(C)]
pub(crate) struct QActionPointer {
    _data: MaybeUninit<[usize; 2]>,
}

unsafe impl ExternType for QActionPointer {
    type Id = type_id!("QActionPointer");
    type Kind = cxx::kind::Trivial;
}

impl std::fmt::Debug for QActionPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QActionPointer").finish_non_exhaustive()
    }
}

#[cxx::bridge]
mod bindings {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/menu/ffi.hpp");

        type QWidget = crate::widgets::QWidget;
        type QString = crate::common::QString;
        type QImage = crate::platform::QImage;
        type QMenuBar;
        type QMenu;
        type QAction;
        type QActionGroup;
        type QActionPointer = super::QActionPointer;

        unsafe fn action_weak(action: *mut QAction) -> QActionPointer;
        fn isNull(self: &QActionPointer) -> bool;

        unsafe fn new_menu_bar(window: *mut QWidget) -> Result<*mut QMenuBar>;
        unsafe fn menu_bar_exists(window: *mut QWidget) -> bool;

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

        fn action_set_icon(action: Pin<&mut QAction>, icon: &QImage) -> Result<()>;
        fn action_clear_icon(action: Pin<&mut QAction>) -> Result<()>;

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

        unsafe fn action_remove_from_parent(action: *mut QAction);
    }
}

pub(crate) use bindings::*;
