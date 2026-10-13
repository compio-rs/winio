use winio_handle::BorrowedMenuContainer;
use winui3::Microsoft::UI::Xaml::Controls::MenuFlyoutItemBase;

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
