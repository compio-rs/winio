use objc2::rc::Retained;
use objc2_foundation::MainThreadMarker;
use objc2_ui_kit::{UIApplication, UIUserInterfaceStyle, UIWindowScene};
use winio_primitive::ColorTheme;

mod canvas;
pub use canvas::*;

mod window;
pub use window::*;

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

mod tab_view;
pub use tab_view::*;

#[cfg(feature = "wgpu")]
mod wgpu;
#[cfg(feature = "wgpu")]
pub use wgpu::*;

// OK to use `keyWindow` because it is for application wide theme detection.
pub fn color_theme() -> crate::Result<ColorTheme> {
    let theme = first_ui_window_scene()?
        .map(|scene| {
            let trait_collection = scene.traitCollection();
            let style = unsafe { trait_collection.userInterfaceStyle() };
            match style {
                UIUserInterfaceStyle::Dark => ColorTheme::Dark,
                _ => ColorTheme::Light,
            }
        })
        .unwrap_or(ColorTheme::Light);
    Ok(theme)
}

pub(crate) fn first_ui_window_scene() -> crate::Result<Option<Retained<UIWindowScene>>> {
    let mtm = MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;
    crate::catch(|| {
        let app = UIApplication::sharedApplication(mtm);
        for scene in app.connectedScenes() {
            if let Ok(scene) = Retained::downcast::<UIWindowScene>(scene) {
                return Some(scene);
            }
        }
        None
    })
}
