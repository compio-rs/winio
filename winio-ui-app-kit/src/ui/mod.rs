use objc2_foundation::{NSUserDefaults, ns_string};
use winio_primitive::ColorTheme;

mod canvas;
pub use canvas::*;

mod window;
pub use window::*;

mod monitor;
pub use monitor::*;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod scroll_bar;
pub use scroll_bar::*;

mod scroll_view;
pub use scroll_view::*;

mod slider;
pub use slider::*;

#[cfg(feature = "media")]
mod media;
#[cfg(feature = "media")]
pub use media::*;

#[cfg(feature = "webview")]
mod webview;
#[cfg(feature = "webview")]
pub use webview::*;

mod accent;
pub use accent::*;

mod tab_view;
pub use tab_view::*;

pub fn color_theme() -> crate::Result<ColorTheme> {
    crate::catch(|| {
        let osx_mode =
            NSUserDefaults::standardUserDefaults().stringForKey(ns_string!("AppleInterfaceStyle"));
        let is_dark = osx_mode
            .map(|mode| mode.isEqualToString(ns_string!("Dark")))
            .unwrap_or_default();
        if is_dark {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    })
}
