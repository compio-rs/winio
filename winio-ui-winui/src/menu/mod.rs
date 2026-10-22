use windows_core::{Interface, Weak};
use winio_handle::BorrowedMenuContainer;
use winui3::Microsoft::UI::Xaml::Controls::{self as MUXC, MenuFlyoutItemBase};

mod bar;
pub use bar::*;

mod bar_item;
pub use bar_item::*;

mod item;
pub use item::*;

mod popup;
pub use popup::*;

mod check;
pub use check::*;

mod radio;
pub use radio::*;

mod separator;
pub use separator::*;

pub(crate) enum MenuParent {
    Flyout(Weak<MUXC::MenuFlyout>),
    SubItem(Weak<MUXC::MenuFlyoutSubItem>),
    BarItem(Weak<MUXC::MenuBarItem>),
}

impl std::fmt::Debug for MenuParent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuParent").finish_non_exhaustive()
    }
}

impl MenuParent {
    pub(crate) fn remove(&self, element: &MenuFlyoutItemBase) -> crate::Result<()> {
        let items = match self {
            Self::Flyout(flyout) => match flyout.upgrade() {
                Some(flyout) => flyout.Items()?,
                None => return Ok(()),
            },
            Self::SubItem(item) => match item.upgrade() {
                Some(item) => item.Items()?,
                None => return Ok(()),
            },
            Self::BarItem(item) => match item.upgrade() {
                Some(item) => item.Items()?,
                None => return Ok(()),
            },
        };
        let mut index = 0;
        if items.IndexOf(element, &mut index)? {
            items.RemoveAt(index)?;
        }
        Ok(())
    }
}

fn append_to_container(
    container: BorrowedMenuContainer,
    element: &MenuFlyoutItemBase,
) -> crate::Result<MenuParent> {
    if let Some(item) = container.as_winui_menu_item() {
        item.Items()?.Append(element)?;
        Ok(MenuParent::SubItem(item.downgrade()?))
    } else if let Some(item) = container.as_winui_menu_bar() {
        item.Items()?.Append(element)?;
        Ok(MenuParent::BarItem(item.downgrade()?))
    } else if let Some(flyout) = container.as_winui_menu_flyout() {
        flyout.Items()?.Append(element)?;
        Ok(MenuParent::Flyout(flyout.downgrade()?))
    } else {
        unreachable!("unsupported menu container type");
    }
}
