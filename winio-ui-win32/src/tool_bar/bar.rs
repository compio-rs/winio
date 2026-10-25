use std::rc::Rc;

use windows_core::WIN32_ERROR;
use windows_sys::Win32::{
    Foundation::ERROR_ALREADY_EXISTS,
    UI::{
        Controls::{
            CCS_NODIVIDER, CCS_TOP, TBSTYLE_FLAT, TBSTYLE_LIST, TBSTYLE_TRANSPARENT,
            TOOLBARCLASSNAMEW,
        },
        WindowsAndMessaging::{IsChild, WS_CHILD, WS_CLIPSIBLINGS, WS_VISIBLE},
    },
};
use winio_handle::AsWindow;

use super::{TOOLBARS, ToolBarData};
use crate::{Result, Widget};

#[derive(Debug)]
pub struct ToolBar {
    data: Rc<ToolBarData>,
}

impl Drop for ToolBar {
    fn drop(&mut self) {
        let _ = TOOLBARS.try_with(|toolbars| {
            toolbars
                .borrow_mut()
                .retain(|toolbar| !Rc::ptr_eq(toolbar, &self.data));
        });
    }
}

impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let parent = parent.as_window().as_win32();
        if TOOLBARS.with_borrow(|toolbars| {
            toolbars
                .iter()
                .any(|data| unsafe { IsChild(parent, data.handle.as_window().as_win32()) != 0 })
        }) {
            return Err(WIN32_ERROR(ERROR_ALREADY_EXISTS).to_hresult().into());
        }
        let handle = Widget::new(
            TOOLBARCLASSNAMEW,
            WS_CHILD
                | WS_VISIBLE
                | WS_CLIPSIBLINGS
                | CCS_TOP as u32
                | CCS_NODIVIDER as u32
                | TBSTYLE_FLAT
                | TBSTYLE_LIST
                | TBSTYLE_TRANSPARENT,
            0,
            parent,
        )?;
        let data = Rc::new(ToolBarData::new(handle));
        data.init_control()?;
        TOOLBARS.with(|toolbars| toolbars.borrow_mut().push(data.clone()));
        Ok(Self { data })
    }

    pub fn is_visible(&self) -> Result<bool> {
        self.data.handle.is_visible()
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.data.handle.set_visible(visible)
    }

    pub(crate) fn data(&self) -> &Rc<ToolBarData> {
        &self.data
    }
}
