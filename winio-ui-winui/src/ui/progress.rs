use inherit_methods_macro::inherit_methods;
use windows::core::Interface;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::Widget;

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    progress_bar: MUXC::ProgressBar,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        let progress_bar = MUXC::ProgressBar::new().unwrap();
        Self {
            handle: Widget::new(parent, progress_bar.cast().unwrap()),
            progress_bar,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let size = self.handle.preferred_size();
        Size::new(0.0, size.height)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn minimum(&self) -> usize {
        self.progress_bar.Minimum().unwrap() as usize
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.progress_bar.SetMinimum(v as _).unwrap();
    }

    pub fn maximum(&self) -> usize {
        self.progress_bar.Maximum().unwrap() as usize
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.progress_bar.SetMaximum(v as _).unwrap()
    }

    pub fn pos(&self) -> usize {
        self.progress_bar.Value().unwrap() as usize
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.progress_bar.SetValue(pos as f64).unwrap();
    }

    pub fn is_indeterminate(&self) -> bool {
        self.progress_bar.IsIndeterminate().unwrap()
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool) {
        self.progress_bar.SetIsIndeterminate(indeterminate).unwrap();
    }
}

winio_handle::impl_as_widget!(Progress, handle);
