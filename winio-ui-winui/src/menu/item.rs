use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};
use winio_pollable::GlobalRuntime;
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Image, Result, menu::append_to_container};

#[derive(Debug)]
pub struct MenuItem {
    on_click: SendWrapper<Rc<Callback>>,
    element: MUXC::MenuFlyoutItem,
    base: MUXC::MenuFlyoutItemBase,
}

impl MenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let element = MUXC::MenuFlyoutItem::new()?;
        let base = element.cast()?;
        append_to_container(parent.as_menu_container(), &base)?;
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
            on_click,
            element,
            base,
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.element.IsEnabled()
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.element.SetIsEnabled(enabled)?;
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.element.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.element.SetText(&HSTRING::from(text.as_ref()))?;
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
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
        self.on_click.wait().await
    }
}

impl AsMenuItem for MenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::winui(&self.base)
    }
}
