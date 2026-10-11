use winio_handle::BorrowedMenuContainer;
use winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutItemBase;

mod menu_bar;
pub use menu_bar::*;

mod menu_bar_item;
pub use menu_bar_item::*;

mod menu_item;
pub use menu_item::*;

mod popup_menu_item;
pub use popup_menu_item::*;

mod check_menu_item;
pub use check_menu_item::*;

mod radio_menu_item;
pub use radio_menu_item::*;

mod separator_menu_item;
pub use separator_menu_item::*;

fn append_to_container(
    container: BorrowedMenuContainer,
    element: &MenuFlyoutItemBase,
) -> crate::Result<()> {
    if let Some(item) = container.as_winui_menu_item() {
        item.Items()?.Append(element)?;
    } else if let Some(bar) = container.as_winui_menu_bar() {
        bar.Items()?.Append(element)?;
    } else {
        unreachable!("unsupported menu container type");
    }
    Ok(())
}
