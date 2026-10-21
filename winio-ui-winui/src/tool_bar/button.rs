use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{GlobalRuntime, Image, Result, tool_bar::ToolBar};

#[derive(Debug)]
pub struct ToolBarButton {
    button: MUXC::AppBarButton,
    on_click: SendWrapper<Rc<Callback>>,
}

impl ToolBarButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let button = MUXC::AppBarButton::new()?;
        parent.element().PrimaryCommands()?.Append(&button)?;
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            button
                .Click(move |_, _| {
                    on_click.signal::<GlobalRuntime>(());
                    Ok(())
                })?
                .forget();
        }
        Ok(Self { button, on_click })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.button.IsEnabled()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.button.SetIsEnabled(enabled)?;
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.button.Label()?.to_string_lossy())
    }

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
        let text = text.as_ref();
        self.button.SetLabel(&HSTRING::from(text))?;
        self.button.SetLabelPosition(if text.is_empty() {
            MUXC::CommandBarLabelPosition::Collapsed
        } else {
            MUXC::CommandBarLabelPosition::Default
        })?;
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
        self.button.SetIcon(icon.as_ref())?;
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}
