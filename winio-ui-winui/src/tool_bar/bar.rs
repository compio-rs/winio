use winio_handle::AsWindow;
use winui3::Microsoft::UI::Xaml::{self as MUX, Controls as MUXC, Visibility};

use crate::Result;

#[derive(Debug)]
pub struct ToolBar {
    element: MUXC::CommandBar,
}

impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let window = parent.as_window().as_winui().clone();
        let element = MUXC::CommandBar::new()?;
        element.SetHorizontalAlignment(MUX::HorizontalAlignment::Stretch)?;
        element.SetDefaultLabelPosition(MUXC::CommandBarDefaultLabelPosition::Right)?;
        element.SetIsDynamicOverflowEnabled(true)?;
        crate::set_toolbar(&window, element.clone())?;
        Ok(Self { element })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.element.Visibility()? == Visibility::Visible)
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.element.SetVisibility(if visible {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        Ok(())
    }

    pub(crate) fn element(&self) -> &MUXC::CommandBar {
        &self.element
    }
}

impl Drop for ToolBar {
    fn drop(&mut self) {
        crate::remove_toolbar(&self.element);
    }
}
