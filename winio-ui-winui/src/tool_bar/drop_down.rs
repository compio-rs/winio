use inherit_methods_macro::inherit_methods;
use windows_core::{HSTRING, Interface};
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{
    Image, Result,
    tool_bar::{ToolBar, ToolBarItem},
};

#[derive(Debug)]
pub struct ToolBarMenuButton {
    button: MUXC::AppBarButton,
    item: ToolBarItem,
    flyout: MUXC::MenuFlyout,
}

#[inherit_methods(from = "self.item")]
impl ToolBarMenuButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let button = MUXC::AppBarButton::new()?;
        let flyout = MUXC::MenuFlyout::new()?;
        button.SetFlyout(&flyout)?;
        parent.element().PrimaryCommands()?.Append(&button)?;
        button.SetLabelPosition(MUXC::CommandBarLabelPosition::Collapsed)?;
        let item = ToolBarItem::new(parent, button.cast()?)?;
        {
            let flyout = flyout.clone();
            let button_for_click = button.clone();
            button
                .Click(move |_, _| {
                    let point = windows::Foundation::Point {
                        X: 0.0,
                        Y: button_for_click.ActualHeight()? as f32,
                    };
                    flyout.ShowAt2(&button_for_click, point)?;
                    Ok(())
                })?
                .forget();
        }
        Ok(Self {
            button,
            item,
            flyout,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&self, enabled: bool) -> Result<()>;

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

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

impl AsMenuContainer for ToolBarMenuButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::winui_menu_flyout(&self.flyout)
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarMenuButton, item);
