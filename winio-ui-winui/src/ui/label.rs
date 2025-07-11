use inherit_methods_macro::inherit_methods;
use windows::core::{HSTRING, Interface};
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Widget, ui::Convertible};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
    label: MUXC::TextBlock,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let label = MUXC::TextBlock::new().unwrap();
        Self {
            handle: Widget::new(parent, label.cast().unwrap()),
            label,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn text(&self) -> String {
        self.label.Text().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.label.SetText(&HSTRING::from(s.as_ref())).unwrap();
    }

    pub fn halign(&self) -> HAlign {
        HAlign::from_native(self.label.TextAlignment().unwrap())
    }

    pub fn set_halign(&mut self, align: HAlign) {
        self.label.SetTextAlignment(align.to_native()).unwrap();
    }
}
