use objc2::rc::Retained;
use objc2_app_kit::{NSBezelStyle, NSImageScaling, NSMenu, NSPopUpButton};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, ns_string};
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{
    Image, Result, catch, from_nsstring,
    tool_bar::{ToolBar, ToolBarItem, refresh_image_position},
};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    item: ToolBarItem,
    button: Retained<NSPopUpButton>,
    menu: Retained<NSMenu>,
}

impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let mtm = parent.data().mtm();

        catch(|| {
            let menu = NSMenu::new(mtm);
            let button = NSPopUpButton::initWithFrame_pullsDown(
                mtm.alloc(),
                NSRect::new(NSPoint::ZERO, NSSize::ZERO),
                true,
            );
            button.setMenu(Some(&menu));
            button.addItemWithTitle(ns_string!(""));
            button.setBezelStyle(NSBezelStyle::Toolbar);
            button.setImageScaling(NSImageScaling::ScaleProportionallyDown);

            button.sizeToFit();
            let item = ToolBarItem::new(parent, &button, "")?;
            Ok(Self { item, button, menu })
        })
        .flatten()
    }

    pub fn is_enabled(&self) -> Result<bool> {
        catch(|| self.button.isEnabled())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        catch(|| self.button.setEnabled(enabled))
    }

    pub fn text(&self) -> Result<String> {
        catch(|| {
            self.menu
                .itemAtIndex(0)
                .map(|item| from_nsstring(&item.title()))
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let text = text.as_ref();
        let title = NSString::from_str(text);
        catch(|| {
            self.button.setTitle(&title);
            if let Some(item) = self.menu.itemAtIndex(0) {
                item.setTitle(&title);
            }
            Ok(())
        })
        .flatten()?;
        self.item.set_label(text)?;
        self.refresh_appearance()
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.nsimage(Some(24.0))?;
                self.button.setImage(Some(&image));
                if let Some(item) = self.menu.itemAtIndex(0) {
                    item.setImage(Some(&image));
                }
            } else {
                self.button.setImage(None);
                if let Some(item) = self.menu.itemAtIndex(0) {
                    item.setImage(None);
                }
            }
            Ok(())
        })
        .flatten()?;
        self.refresh_appearance()
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }

    fn refresh_appearance(&self) -> Result<()> {
        catch(|| {
            refresh_image_position(&self.button);
            self.button.sizeToFit();
            Ok(())
        })
        .flatten()
    }
}

impl AsMenuContainer for ToolBarDropDownButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::app_kit(&self.menu)
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
