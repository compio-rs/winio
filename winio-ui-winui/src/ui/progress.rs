use inherit_methods_macro::inherit_methods;
use windows::core::Interface;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as WUXC;

use crate::Widget;

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    progress_bar: WUXC::ProgressBar,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        let progress_bar = WUXC::ProgressBar::new().unwrap();
        Self {
            handle: Widget::new(parent, progress_bar.cast().unwrap()),
            progress_bar,
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

    pub fn range(&self) -> (usize, usize) {
        let min = self.progress_bar.Minimum().unwrap() as usize;
        let max = self.progress_bar.Maximum().unwrap() as usize;
        (min, max)
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.progress_bar.SetMinimum(min as f64).unwrap();
        self.progress_bar.SetMaximum(max as f64).unwrap();
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
