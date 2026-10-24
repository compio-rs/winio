use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_foundation::NSString;
use objc2_ui_kit::{UIButton, UIButtonType, UIControlState, UIFont};
use winio_handle::{AsContainer, AsMenuContainer, BorrowedMenuContainer};
use winio_primitive::{Point, Size};

use crate::{
    Image, Result, catch, from_nsstring,
    menu::{MenuNodeRef, build_bar_menu},
    widgets::Widget,
};

#[derive(Debug)]
pub struct MenuButton {
    handle: Widget,
    view: Retained<UIButton>,
    menu: MenuNodeRef,
}

#[inherit_methods(from = "self.handle")]
impl MenuButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| {
            let menu = MenuNodeRef::root();
            let view = UIButton::buttonWithType(UIButtonType::System, mtm);
            view.setShowsMenuAsPrimaryAction(true);
            view.setMenu(Some(&build_bar_menu(menu.key(), mtm)));

            let handle =
                Widget::from_uiview(parent, unsafe { Retained::cast_unchecked(view.clone()) })?;
            Ok(Self { handle, view, menu })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        catch(|| {
            self.view
                .titleForState(UIControlState::Normal)
                .as_deref()
                .map(from_nsstring)
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let title = NSString::from_str(s.as_ref());
        catch(|| {
            self.view
                .setTitle_forState(Some(&title), UIControlState::Normal);
            self.view.invalidateIntrinsicContentSize();
        })
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.uiimage(Some(UIFont::systemFontSize()))?;
                self.view
                    .setImage_forState(Some(&image), UIControlState::Normal);
            } else {
                self.view.setImage_forState(None, UIControlState::Normal);
            }
            self.view.invalidateIntrinsicContentSize();
            Ok(())
        })
        .flatten()
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

impl AsMenuContainer for MenuButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::ui_kit(self.menu.key())
    }
}

winio_handle::impl_as_widget!(MenuButton, handle);
