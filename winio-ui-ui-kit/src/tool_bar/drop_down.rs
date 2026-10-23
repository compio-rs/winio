use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_ui_kit::{UIBarButtonItem, UIButton, UIButtonType, UIControlState};
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{
    Image, Result, catch, from_nsstring,
    menu::{MenuNodeRef, build_bar_menu},
    tool_bar::{ToolBar, ToolBarItem},
};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    item: ToolBarItem,
    button: Retained<UIButton>,
    menu: MenuNodeRef,
}

impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let mtm = parent.data().mtm();

        catch(|| {
            let menu = MenuNodeRef::root();
            let button = UIButton::buttonWithType(UIButtonType::System, mtm);
            button.setShowsMenuAsPrimaryAction(true);
            button.setMenu(Some(&build_bar_menu(menu.key(), mtm)));

            let bar_item = UIBarButtonItem::initWithCustomView(mtm.alloc(), &button);
            let item = ToolBarItem::new(parent, bar_item)?;
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
            self.button
                .titleForState(UIControlState::Normal)
                .as_deref()
                .map(from_nsstring)
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let title = NSString::from_str(text.as_ref());
        catch(|| {
            self.button
                .setTitle_forState(Some(&title), UIControlState::Normal);
            self.button.invalidateIntrinsicContentSize();
        })
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.uiimage(Some(24.0))?;
                self.button
                    .setImage_forState(Some(&image), UIControlState::Normal);
            } else {
                self.button.setImage_forState(None, UIControlState::Normal);
            }
            self.button.invalidateIntrinsicContentSize();
            Ok(())
        })
        .flatten()
    }

    pub async fn wait_click(&self) {
        std::future::pending().await
    }
}

impl AsMenuContainer for ToolBarDropDownButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::ui_kit(self.menu.key())
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
