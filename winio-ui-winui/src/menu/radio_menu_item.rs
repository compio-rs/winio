use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};
use winio_pollable::GlobalRuntime;
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, menu::append_to_container};

#[derive(Debug)]
pub struct RadioMenuItem {
    on_click: SendWrapper<Rc<Callback>>,
    element: MUXC::RadioMenuFlyoutItem,
    base: MUXC::MenuFlyoutItemBase,
}

impl RadioMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let element = MUXC::RadioMenuFlyoutItem::new()?;
        let base = element.cast()?;
        let container = parent.as_menu_container();
        let group = if let Some(item) = container.as_winui_menu_item() {
            item.as_raw() as usize
        } else if let Some(bar) = container.as_winui_menu_bar() {
            bar.as_raw() as usize
        } else {
            unreachable!("unsupported menu container type");
        };
        element.SetGroupName(&HSTRING::from(format!("winio-radio-{group}")))?;
        append_to_container(container, &base)?;
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

    pub fn is_checked(&self) -> Result<bool> {
        self.element.IsChecked()
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        self.element.SetIsChecked(checked)?;
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

impl AsMenuItem for RadioMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::winui(&self.base)
    }
}
