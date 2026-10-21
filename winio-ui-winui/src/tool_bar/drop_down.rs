use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, Controls::Primitives as MUXCP};

use crate::{
    GlobalRuntime, Image, Result,
    tool_bar::{ToolBar, ToolBarItem},
};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    button: MUXC::AppBarButton,
    item: ToolBarItem,
    flyout: MUXC::MenuFlyout,
    on_click: SendWrapper<Rc<Callback>>,
}

#[inherit_methods(from = "self.item")]
impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let button = MUXC::AppBarButton::new()?;
        let flyout = MUXC::MenuFlyout::new()?;
        button.SetFlyout(&flyout.cast::<MUXCP::FlyoutBase>()?)?;
        parent.element().PrimaryCommands()?.Append(&button)?;
        let item = ToolBarItem::new(button.cast()?)?;
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            let flyout = flyout.clone();
            let button_for_click = button.clone();
            button
                .Click(move |_, _| {
                    let point = windows::Foundation::Point {
                        X: 0.0,
                        Y: button_for_click.ActualHeight()? as f32,
                    };
                    flyout.ShowAt2(&button_for_click, point)?;
                    on_click.signal::<GlobalRuntime>(());
                    Ok(())
                })?
                .forget();
        }
        Ok(Self {
            button,
            item,
            flyout,
            on_click,
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

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

impl AsMenuContainer for ToolBarDropDownButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::winui_menu_flyout(&self.flyout)
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
