use winio_callback::Callback;
use winio_handle::{AsMenuItem, BorrowedMenuItem};

use crate::{
    GlobalRuntime, Image, Result,
    menu::{
        RawPtr,
        ffi::{QAction, action_clear_icon, action_connect_triggered, action_set_icon},
    },
};

#[derive(Debug)]
pub(crate) struct ActionHandle {
    action: RawPtr<QAction>,
}

impl ActionHandle {
    pub(crate) fn new(action: *mut QAction) -> Self {
        Self {
            action: RawPtr(action),
        }
    }

    pub(crate) fn action_ptr(&self) -> *mut QAction {
        self.action.0
    }

    pub(crate) fn set_checkable(&mut self, checkable: bool) -> Result<()> {
        unsafe { self.action.pin_mut() }.setCheckable(checkable)?;
        Ok(())
    }

    pub(crate) fn is_enabled(&self) -> Result<bool> {
        Ok(unsafe { self.action.as_ref() }.isEnabled()?)
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        unsafe { self.action.pin_mut() }.setEnabled(enabled)?;
        Ok(())
    }

    pub(crate) fn text(&self) -> Result<String> {
        Ok(unsafe { self.action.as_ref() }.text()?.try_into()?)
    }

    pub(crate) fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        unsafe { self.action.pin_mut() }.setText(&s.as_ref().try_into()?)?;
        Ok(())
    }

    pub(crate) fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        unsafe {
            let action = self.action.pin_mut();
            match icon {
                Some(icon) => action_set_icon(action, icon.as_qimage())?,
                None => action_clear_icon(action)?,
            }
        }
        Ok(())
    }

    pub(crate) fn is_checked(&self) -> Result<bool> {
        Ok(unsafe { self.action.as_ref() }.isChecked()?)
    }

    pub(crate) fn set_checked(&mut self, checked: bool) -> Result<()> {
        unsafe { self.action.pin_mut() }.setChecked(checked)?;
        Ok(())
    }
}

impl AsMenuItem for ActionHandle {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        unsafe { BorrowedMenuItem::qt(self.action.0) }
    }
}

#[derive(Debug)]
pub(crate) struct ActionClick {
    on_click: Box<Callback>,
}

impl ActionClick {
    pub(crate) fn new(action: *mut QAction) -> Result<Self> {
        let on_click = Box::new(Callback::new());
        let mut action = RawPtr(action);
        unsafe {
            action_connect_triggered(
                action.pin_mut(),
                Self::on_click,
                on_click.as_ref() as *const _ as _,
            )?
        }
        Ok(Self { on_click })
    }

    fn on_click(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub(crate) async fn wait_click(&self) {
        self.on_click.wait().await
    }
}
