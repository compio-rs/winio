use objc2::{MainThreadOnly, rc::Retained};
use objc2_foundation::NSString;
use objc2_ui_kit::{UIButton, UIButtonType, UIControlState};

use crate::{
    Result, catch, from_nsstring,
    menu::{MenuBar, MenuNodeRef, build_bar_menu},
};

#[derive(Debug)]
pub struct MenuBarItem {
    node: MenuNodeRef,
    button: Retained<UIButton>,
}

impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        let stack = parent.stack();
        let mtm = stack.mtm();
        catch(|| {
            let node = MenuNodeRef::root();
            let button = UIButton::buttonWithType(UIButtonType::System, mtm);
            button.setShowsMenuAsPrimaryAction(true);
            button.setMenu(Some(&build_bar_menu(node.key(), mtm)));
            stack.addArrangedSubview(&button);

            Ok(Self { node, button })
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

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let title = NSString::from_str(s.as_ref());
        catch(|| {
            self.button
                .setTitle_forState(Some(&title), UIControlState::Normal)
        })
    }
}

impl Drop for MenuBarItem {
    fn drop(&mut self) {
        self.button.removeFromSuperview();
    }
}

winio_handle::impl_as_menu_container!(MenuBarItem, node);
