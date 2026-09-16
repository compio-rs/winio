use inherit_methods_macro::inherit_methods;
use windows_core::{HSTRING, Interface};
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Font, HAlign, Point, Size};
use winui3::Microsoft::UI::Xaml::{self as MUX, Controls as MUXC, TextWrapping};

use crate::{
    Image, Result, Widget,
    widgets::{Convertible, font_to_text_block, text_block_to_font},
};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
    widget: LabelImpl,
}

#[derive(Debug)]
enum LabelImpl {
    Label(MUXC::TextBlock),
    Image(MUXC::Image),
}

impl LabelImpl {
    fn element(&self) -> Result<MUX::FrameworkElement> {
        Ok(match self {
            Self::Label(widget) => widget.cast()?,
            Self::Image(widget) => widget.cast()?,
        })
    }

    fn as_label(&self) -> Option<&MUXC::TextBlock> {
        match self {
            Self::Label(widget) => Some(widget),
            Self::Image(_) => None,
        }
    }

    fn as_label_mut(&mut self) -> Option<&mut MUXC::TextBlock> {
        match self {
            Self::Label(widget) => Some(widget),
            Self::Image(_) => None,
        }
    }
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let label = MUXC::TextBlock::new()?;
        label.SetTextWrapping(TextWrapping::Wrap)?;
        let handle = Widget::new(parent, label.cast()?)?;
        Ok(Self {
            handle,
            widget: LabelImpl::Label(label),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let mut size = self.handle.preferred_size()?;
        size.width += 1.0;
        size.height += 1.0;
        Ok(size)
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(match &self.widget {
            LabelImpl::Label(widget) => widget.Text()?.to_string_lossy(),
            LabelImpl::Image(_) => String::new(),
        })
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        let widget = match &mut self.widget {
            LabelImpl::Label(widget) => {
                widget.SetText(&HSTRING::from(s))?;
                None
            }
            LabelImpl::Image(_) => {
                let widget = MUXC::TextBlock::new()?;
                widget.SetTextWrapping(TextWrapping::Wrap)?;
                widget.SetText(&HSTRING::from(s))?;
                Some(LabelImpl::Label(widget))
            }
        };
        if let Some(widget) = widget {
            self.replace_widget(widget)?;
        }
        Ok(())
    }

    pub fn set_image(&mut self, image: &Image) -> Result<()> {
        let source = image.source()?;
        let widget = match &mut self.widget {
            LabelImpl::Image(widget) => {
                widget.SetSource(&source)?;
                None
            }
            LabelImpl::Label(_) => {
                let widget = MUXC::Image::new()?;
                widget.SetSource(&source)?;
                Some(LabelImpl::Image(widget))
            }
        };
        if let Some(widget) = widget {
            self.replace_widget(widget)?;
        }
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        let Some(widget) = self.widget.as_label() else {
            return Ok(HAlign::Center);
        };
        Ok(HAlign::from_native(widget.TextAlignment()?))
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        if let Some(widget) = self.widget.as_label_mut() {
            widget.SetTextAlignment(align.to_native())?;
        }
        Ok(())
    }

    pub fn font(&self) -> Result<Font> {
        let Some(widget) = self.widget.as_label() else {
            return Ok(Font::default());
        };
        text_block_to_font(widget)
    }

    pub fn set_font(&mut self, font: Font) -> Result<()> {
        if let Some(widget) = self.widget.as_label_mut() {
            font_to_text_block(widget, &font)?;
        }
        Ok(())
    }

    fn replace_widget(&mut self, widget: LabelImpl) -> Result<()> {
        let parent = self.handle.parent()?;
        let location = self.handle.loc()?;
        let visible = self.handle.is_visible()?;
        let enabled = self.handle.is_enabled()?;
        let tooltip = self.handle.tooltip()?;
        let mut handle = Widget::new(BorrowedContainer::winui(&parent), widget.element()?)?;
        handle.set_loc(location)?;
        handle.set_visible(visible)?;
        handle.set_enabled(enabled)?;
        if !tooltip.is_empty() {
            handle.set_tooltip(tooltip)?;
        }
        self.handle = handle;
        self.widget = widget;
        Ok(())
    }
}

winio_handle::impl_as_widget!(Label, handle);
