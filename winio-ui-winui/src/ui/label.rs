use inherit_methods_macro::inherit_methods;
use windows::core::{HSTRING, Interface};
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, TextWrapping};

use crate::{Result, Widget, ui::Convertible};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
    label: MUXC::TextBlock,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let label = MUXC::TextBlock::new()?;
        label.SetTextWrapping(TextWrapping::Wrap)?;
        Ok(Self {
            handle: Widget::new(parent, label.cast()?)?,
            label,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(self.label.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.label.SetText(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        Ok(HAlign::from_native(self.label.TextAlignment()?))
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        self.label.SetTextAlignment(align.to_native())?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(Label, handle);
