use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSFont, NSImageScaling, NSMenu, NSPopUpButton};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, ns_string};
use winio_handle::{AsContainer, AsMenuContainer, BorrowedMenuContainer};
use winio_primitive::{Point, Size};

use crate::{
    Image, Result, catch, from_nsstring, tool_bar::refresh_image_position, widgets::Widget,
};

#[derive(Debug)]
pub struct MenuButton {
    handle: Widget,
    button: Retained<NSPopUpButton>,
    menu: Retained<NSMenu>,
}

#[inherit_methods(from = "self.handle")]
impl MenuButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| {
            let menu = NSMenu::new(mtm);
            let button = NSPopUpButton::initWithFrame_pullsDown(
                mtm.alloc(),
                NSRect::new(NSPoint::ZERO, NSSize::ZERO),
                true,
            );
            button.setMenu(Some(&menu));
            button.addItemWithTitle(ns_string!(""));
            button.setImageScaling(NSImageScaling::ScaleProportionallyDown);

            let handle =
                Widget::from_nsview(parent, unsafe { Retained::cast_unchecked(button.clone()) })?;
            Ok(Self {
                handle,
                button,
                menu,
            })
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
            self.menu
                .itemAtIndex(0)
                .map(|item| from_nsstring(&item.title()))
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let title = NSString::from_str(s.as_ref());
        catch(|| {
            self.button.setTitle(&title);
            if let Some(item) = self.menu.itemAtIndex(0) {
                item.setTitle(&title);
            }
        })?;
        self.refresh_appearance()
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.nsimage(Some(NSFont::systemFontSize()))?;
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
        catch(|| refresh_image_position(&self.button))
    }
}

impl AsMenuContainer for MenuButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::app_kit(&self.menu)
    }
}

winio_handle::impl_as_widget!(MenuButton, handle);
