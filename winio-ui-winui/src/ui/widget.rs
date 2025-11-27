use compio_log::error;
use windows::core::Interface;
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{self as MUX, Controls as MUXC};

use crate::{Result, ui::Convertible};

#[derive(Debug)]
pub(crate) struct Widget {
    handle: MUX::FrameworkElement,
    pub(crate) parent: MUXC::Canvas,
}

impl Widget {
    pub fn new(parent: impl AsContainer, handle: MUX::FrameworkElement) -> Result<Self> {
        handle.SetHorizontalAlignment(MUX::HorizontalAlignment::Center)?;
        handle.SetVerticalAlignment(MUX::VerticalAlignment::Center)?;
        let parent = parent.as_container();
        let canvas = parent.as_winui();
        canvas.Children()?.Append(&handle)?;
        canvas.Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())?;
        Ok(Self {
            handle,
            parent: canvas.clone(),
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.handle.Visibility()? == MUX::Visibility::Visible)
    }

    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.handle.SetVisibility(if visible {
            MUX::Visibility::Visible
        } else {
            MUX::Visibility::Collapsed
        })?;
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        if let Ok(handle) = self.handle.cast::<MUXC::Control>() {
            Ok(handle.IsEnabled()?)
        } else {
            Ok(true)
        }
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        if let Ok(handle) = self.handle.cast::<MUXC::Control>() {
            handle.SetIsEnabled(enabled)?;
        }
        Ok(())
    }

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::from_native(self.handle.MeasureOverride(
            Size::new(f64::INFINITY, f64::INFINITY).to_native(),
        )?))
    }

    pub fn min_size(&self) -> Result<Size> {
        let width = self.handle.MinWidth()?;
        let height = self.handle.MinHeight()?;
        Ok(Size::new(width, height))
    }

    pub fn loc(&self) -> Result<Point> {
        let left = MUXC::Canvas::GetLeft(&self.handle)?;
        let top = MUXC::Canvas::GetTop(&self.handle)?;
        Ok(Point::new(left, top))
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        MUXC::Canvas::SetLeft(&self.handle, p.x)?;
        MUXC::Canvas::SetTop(&self.handle, p.y)?;
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let width = self.handle.Width()?;
        let height = self.handle.Height()?;
        Ok(Size::new(width, height))
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.SetWidth(v.width)?;
        self.handle.SetHeight(v.height)?;
        Ok(())
    }

    pub fn tooltip(&self) -> Result<String> {
        Ok(MUXC::ToolTipService::GetToolTip(&self.handle)
            .and_then(|w| w.cast::<MUXC::TextBlock>())
            .and_then(|w| w.Text())
            .map(|s| s.to_string_lossy())
            .unwrap_or_default())
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        let text = match MUXC::ToolTipService::GetToolTip(&self.handle)
            .and_then(|w| w.cast::<MUXC::TextBlock>())
        {
            Ok(text) => text,
            Err(_) => {
                let text = MUXC::TextBlock::new()?;
                MUXC::ToolTipService::SetToolTip(&self.handle, &text)?;
                text
            }
        };
        text.SetText(&windows::core::HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    fn drop_impl(&mut self) -> Result<()> {
        let children = self.parent.Children()?;
        let mut index = 0;
        if children.IndexOf(&self.handle, &mut index).is_ok() {
            children.RemoveAt(index as _)?;
        }
        Ok(())
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        match self.drop_impl() {
            Ok(()) => {}
            Err(_e) => {
                error!("Widget drop: {_e:?}");
            }
        }
    }
}

impl AsRawWidget for Widget {
    fn as_raw_widget(&self) -> RawWidget {
        RawWidget::WinUI(self.handle.clone())
    }
}
