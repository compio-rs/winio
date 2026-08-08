use std::cell::Cell;

use gtk4::pango::{
    AttrFontDesc, AttrList, FontDescription, SCALE as PANGO_SCALE, Style, Weight,
};
use winio_primitive::{ColorTheme, Font};

thread_local! {
    pub(crate) static COLOR_THEME: Cell<Option<ColorTheme>> = const { Cell::new(None) };
}

pub fn color_theme() -> crate::Result<ColorTheme> {
    COLOR_THEME.get().ok_or(crate::Error::NoColorTheme)
}

/// Convert a [`Font`] to a [`FontDescription`].
pub(crate) fn font_to_desc(font: &Font) -> FontDescription {
    let mut desc = FontDescription::new();
    desc.set_family(&font.family);
    desc.set_size((font.size * PANGO_SCALE as f64) as i32);
    desc.set_style(if font.italic {
        Style::Italic
    } else {
        Style::Normal
    });
    desc.set_weight(if font.bold {
        Weight::Bold
    } else {
        Weight::Normal
    });
    desc
}

/// Convert a [`FontDescription`] to a [`Font`].
pub(crate) fn desc_to_font(desc: &FontDescription) -> Font {
    Font {
        family: desc.family().map(|s| s.to_string()).unwrap_or_default(),
        size: desc.size() as f64 / PANGO_SCALE as f64,
        italic: desc.style() == Style::Italic,
        bold: desc.weight() == Weight::Bold,
    }
}

/// Extract the [`FontDescription`] from an attribute list, if any.
pub(crate) fn font_desc_from_attrs(attr_list: Option<&AttrList>) -> Option<FontDescription> {
    attr_list.and_then(|attr_list| {
        attr_list
            .iterator()
            .attrs()
            .iter()
            .find_map(|attr| attr.downcast_ref::<AttrFontDesc>().map(|d| d.desc()))
    })
}

mod window;
pub use window::*;

mod canvas;
pub use canvas::*;

mod widget;
pub(crate) use widget::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod link_label;
pub use link_label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod check_box;
pub use check_box::*;

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
