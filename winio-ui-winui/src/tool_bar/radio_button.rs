use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, Controls::Primitives as MUXCP};

use crate::{GlobalRuntime, Image, Result, tool_bar::ToolBar};

#[derive(Debug)]
pub struct ToolBarRadioButton {
    element: MUXC::AppBarToggleButton,
    toggle: MUXCP::ToggleButton,
    on_click: SendWrapper<Rc<Callback>>,
}

impl ToolBarRadioButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let element = MUXC::AppBarToggleButton::new()?;
        parent.element().PrimaryCommands()?.Append(&element)?;
        let toggle = element.cast()?;
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            element
                .Click(move |_, _| {
                    on_click.signal::<GlobalRuntime>(());
                    Ok(())
                })?
                .forget();
        }
        Ok(Self {
            element,
            toggle,
            on_click,
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.element.IsEnabled()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.element.SetIsEnabled(enabled)?;
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.element.Label()?.to_string_lossy())
    }

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
        let text = text.as_ref();
        self.element.SetLabel(&HSTRING::from(text))?;
        self.element.SetLabelPosition(if text.is_empty() {
            MUXC::CommandBarLabelPosition::Collapsed
        } else {
            MUXC::CommandBarLabelPosition::Default
        })?;
        Ok(())
    }

    pub fn is_checked(&self) -> Result<bool> {
        self.toggle.IsChecked()
    }

    pub fn set_checked(&self, checked: bool) -> Result<()> {
        self.toggle.SetIsChecked(Some(checked))?;
        Ok(())
    }

    pub fn set_icon(&self, icon: Option<&Image>) -> Result<()> {
        let icon = match icon {
            Some(icon) => {
                let image_icon = MUXC::ImageIcon::new()?;
                image_icon.SetSource(icon.as_ref())?;
                Some(image_icon.cast::<MUXC::IconElement>()?)
            }
            None => None,
        };
        self.element.SetIcon(icon.as_ref())?;
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
        let _ = self.set_checked(true);
    }
}
