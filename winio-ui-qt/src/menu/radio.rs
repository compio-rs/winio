use std::{fmt::Debug, mem::ManuallyDrop};

use cxx::UniquePtr;
use inherit_methods_macro::inherit_methods;
use winio_handle::AsMenuContainer;

use crate::{
    Result,
    menu::{
        ActionClick, ActionHandle, RawPtr,
        ffi::{QAction, QActionGroup, QMenu, menu_add_action, new_action, new_action_group},
    },
};

pub struct RadioMenuItem {
    action: ActionHandle,
    click: ActionClick,
    #[allow(dead_code)]
    group: ManuallyDrop<UniquePtr<QActionGroup>>,
    another_action: RawPtr<QAction>,
}

impl Debug for RadioMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadioMenuItem").finish_non_exhaustive()
    }
}

#[inherit_methods(from = "self.action")]
impl RadioMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let menu = parent.as_menu_container().as_qt::<QMenu>();
        let action = unsafe { menu_add_action(menu) }?;
        unsafe {
            let mut action = RawPtr(action);
            action.pin_mut().setCheckable(true)?;
        }
        let another_action = unsafe { new_action(menu) }?;
        unsafe {
            let mut another_action = RawPtr(another_action);
            another_action.pin_mut().setCheckable(true)?;
        }
        let mut group = unsafe { new_action_group(menu) }?;
        group.pin_mut().setExclusive(true)?;
        unsafe {
            group.pin_mut().addAction(action)?;
            group.pin_mut().addAction(another_action)?;
        }
        Ok(Self {
            action: ActionHandle::new(action),
            click: ActionClick::new(action)?,
            group: ManuallyDrop::new(group),
            another_action: RawPtr(another_action),
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        if checked {
            self.action.set_checked(true)?;
        } else {
            unsafe {
                self.another_action.pin_mut().setChecked(true)?;
            }
        }
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.click.wait_click().await
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, action);
