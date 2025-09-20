//! ELM widgets.

use winio_primitive::{Point, Size};

fn approx_eq_point(p1: Point, p2: Point) -> bool {
    approx_eq(p1.x, p2.x) && approx_eq(p1.y, p2.y)
}

fn approx_eq_size(s1: Size, s2: Size) -> bool {
    approx_eq(s1.width, s2.width) && approx_eq(s1.height, s2.height)
}

fn approx_eq(f1: f64, f2: f64) -> bool {
    (f1 - f2).abs() < 1.0
}

mod window;
pub use window::*;

mod view;
pub use view::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod canvas;
pub use canvas::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;

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

mod tooltip;
pub use tooltip::*;

mod tab_view;
pub use tab_view::*;
