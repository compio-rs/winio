use inherit_methods_macro::inherit_methods;
use windows::core::Interface;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, Widget};

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    progress_bar: MUXC::ProgressBar,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let progress_bar = MUXC::ProgressBar::new()?;
        Ok(Self {
            handle: Widget::new(parent, progress_bar.cast()?)?,
            progress_bar,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let size = self.handle.preferred_size()?;
        Ok(Size::new(0.0, size.height))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.progress_bar.Minimum()? as usize)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        self.progress_bar.SetMinimum(v as _)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.progress_bar.Maximum()? as usize)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        self.progress_bar.SetMaximum(v as _)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        Ok(self.progress_bar.Value()? as usize)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        self.progress_bar.SetValue(pos as f64)?;
        Ok(())
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        self.progress_bar.IsIndeterminate()
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool) -> Result<()> {
        self.progress_bar.SetIsIndeterminate(indeterminate)?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(Progress, handle);
