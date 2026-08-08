use objc2::rc::Retained;
use objc2_app_kit::{NSFont, NSFontDescriptorSymbolicTraits};
use objc2_foundation::{NSUserDefaults, NSString, ns_string};
use winio_primitive::{ColorTheme, Font};

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

mod tab_view;
pub use tab_view::*;

#[cfg(feature = "wgpu")]
mod wgpu;
#[cfg(feature = "wgpu")]
pub use wgpu::*;

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

/// Convert an [`NSFont`] to a [`Font`].
pub(crate) fn nsfont_to_font(font: &NSFont) -> Font {
    let traits = font.fontDescriptor().symbolicTraits();
    Font {
        family: font
            .familyName()
            .map(|s| crate::from_nsstring(&s))
            .unwrap_or_default(),
        size: font.pointSize(),
        bold: traits.contains(NSFontDescriptorSymbolicTraits::TraitBold),
        italic: traits.contains(NSFontDescriptorSymbolicTraits::TraitItalic),
    }
}

/// Convert a [`Font`] to an [`NSFont`].
pub(crate) fn font_to_nsfont(font: &Font) -> Retained<NSFont> {
    let base = NSFont::fontWithName_size(&NSString::from_str(&font.family), font.size)
        .unwrap_or_else(|| NSFont::systemFontOfSize(font.size));
    let mut traits = NSFontDescriptorSymbolicTraits::empty();
    if font.bold {
        traits |= NSFontDescriptorSymbolicTraits::TraitBold;
    }
    if font.italic {
        traits |= NSFontDescriptorSymbolicTraits::TraitItalic;
    }
    let desc = base.fontDescriptor().fontDescriptorWithSymbolicTraits(traits);
    NSFont::fontWithDescriptor_size(&desc, font.size).unwrap_or(base)
}
