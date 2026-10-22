use windows_core::{Interface, Weak};
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::Result;

mod bar;
pub use bar::*;

mod button;
pub use button::*;

mod check_box;
pub use check_box::*;

mod drop_down;
pub use drop_down::*;

mod radio_button;
pub use radio_button::*;

mod separator;
pub use separator::*;

pub(crate) struct ToolBarItem {
    bar: Weak<MUXC::CommandBar>,
    handle: MUXC::ICommandBarElement,
}

impl std::fmt::Debug for ToolBarItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolBarItem").finish_non_exhaustive()
    }
}

impl ToolBarItem {
    pub fn new(parent: &ToolBar, handle: MUXC::ICommandBarElement) -> Result<Self> {
        Ok(Self {
            bar: parent.element().downgrade()?,
            handle,
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        let control = self.handle.cast::<MUXC::Control>()?;
        control.IsEnabled()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let control = self.handle.cast::<MUXC::Control>()?;
        control.SetIsEnabled(enabled)?;
        Ok(())
    }
}

impl Drop for ToolBarItem {
    fn drop(&mut self) {
        let Some(bar) = self.bar.upgrade() else {
            return;
        };
        let Ok(items) = bar.PrimaryCommands() else {
            return;
        };
        let mut index = 0;
        if let Ok(true) = items.IndexOf(&self.handle, &mut index) {
            let _ = items.RemoveAt(index);
        }
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::winui(&self.handle)
    }
}
