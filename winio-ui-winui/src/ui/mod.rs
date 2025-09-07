use windows::{
    Foundation::{IReference, PropertyValue},
    Graphics::{PointInt32, SizeInt32},
    core::{HSTRING, Interface, RuntimeType},
};
use winio_primitive::{ColorTheme, HAlign, Orient, Point, Size};
pub use winio_ui_windows_common::{
    CustomButton, FileBox, FileFilter, MessageBox, accent_color, monitor_get_all,
};
use winui3::Microsoft::UI::Xaml::{Application, Controls::Orientation, TextAlignment};

trait Convertible<T> {
    fn from_native(native: T) -> Self;
    fn to_native(self) -> T;
}

impl Convertible<PointInt32> for Point {
    fn from_native(native: PointInt32) -> Self {
        Point::new(native.X as _, native.Y as _)
    }

    fn to_native(self) -> PointInt32 {
        PointInt32 {
            X: self.x as _,
            Y: self.y as _,
        }
    }
}

impl Convertible<SizeInt32> for Size {
    fn from_native(native: SizeInt32) -> Self {
        Size::new(native.Width as _, native.Height as _)
    }

    fn to_native(self) -> SizeInt32 {
        SizeInt32 {
            Width: self.width as _,
            Height: self.height as _,
        }
    }
}

impl Convertible<windows::Foundation::Size> for Size {
    fn from_native(native: windows::Foundation::Size) -> Self {
        Size::new(native.Width as _, native.Height as _)
    }

    fn to_native(self) -> windows::Foundation::Size {
        windows::Foundation::Size {
            Width: self.width as _,
            Height: self.height as _,
        }
    }
}

impl Convertible<windows::Foundation::Point> for Point {
    fn from_native(native: windows::Foundation::Point) -> Self {
        Point::new(native.X as _, native.Y as _)
    }

    fn to_native(self) -> windows::Foundation::Point {
        windows::Foundation::Point {
            X: self.x as _,
            Y: self.y as _,
        }
    }
}

impl Convertible<TextAlignment> for HAlign {
    fn from_native(native: TextAlignment) -> Self {
        match native {
            TextAlignment::Left => HAlign::Left,
            TextAlignment::Center => HAlign::Center,
            TextAlignment::Right => HAlign::Right,
            TextAlignment::Justify => HAlign::Stretch,
            _ => HAlign::Left, // Default to Left if unknown
        }
    }

    fn to_native(self) -> TextAlignment {
        match self {
            HAlign::Left => TextAlignment::Left,
            HAlign::Center => TextAlignment::Center,
            HAlign::Right => TextAlignment::Right,
            HAlign::Stretch => TextAlignment::Justify,
        }
    }
}

impl Convertible<Orientation> for Orient {
    fn from_native(native: Orientation) -> Self {
        match native {
            Orientation::Horizontal => Orient::Horizontal,
            Orientation::Vertical => Orient::Vertical,
            _ => unreachable!(),
        }
    }

    fn to_native(self) -> Orientation {
        match self {
            Orient::Horizontal => Orientation::Horizontal,
            Orient::Vertical => Orientation::Vertical,
        }
    }
}

trait ToIReference: RuntimeType {
    fn to_reference(&self) -> IReference<Self>;
}

impl ToIReference for HSTRING {
    fn to_reference(&self) -> IReference<Self> {
        PropertyValue::CreateString(self).unwrap().cast().unwrap()
    }
}

impl ToIReference for bool {
    fn to_reference(&self) -> IReference<Self> {
        PropertyValue::CreateBoolean(*self).unwrap().cast().unwrap()
    }
}

pub fn color_theme() -> ColorTheme {
    match Application::Current().unwrap().RequestedTheme().unwrap().0 {
        1 => ColorTheme::Dark,
        _ => ColorTheme::Light,
    }
}

mod window;
pub use window::*;

mod widget;
pub(crate) use widget::*;

mod button;
pub use button::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;

mod edit;
pub use edit::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod canvas;
pub use canvas::*;

mod scroll_bar;
pub use scroll_bar::*;

mod slider;
pub use slider::*;

mod tooltip;
pub use tooltip::*;

#[cfg(feature = "media")]
mod media;
#[cfg(feature = "media")]
pub use media::*;
