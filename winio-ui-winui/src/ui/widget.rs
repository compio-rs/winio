use windows::core::Interface;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::{Xaml as MUX, Xaml::Controls as MUXC};

use crate::ui::Convertible;

#[derive(Debug)]
pub(crate) struct Widget {
    handle: MUX::FrameworkElement,
}

impl Widget {
    pub fn new(parent: impl AsWindow, handle: MUX::FrameworkElement) -> Self {
        let parent = parent.as_window();
        let window = parent.as_winui();
        let canvas = window.Content().unwrap().cast::<MUXC::Canvas>().unwrap();
        canvas.Children().unwrap().Append(&handle).unwrap();
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.Visibility().unwrap() == MUX::Visibility::Visible
    }

    pub fn set_visible(&self, visible: bool) {
        self.handle
            .SetVisibility(if visible {
                MUX::Visibility::Visible
            } else {
                MUX::Visibility::Collapsed
            })
            .unwrap();
    }

    pub fn is_enabled(&self) -> bool {
        if let Ok(handle) = self.handle.cast::<MUXC::Control>() {
            handle.IsEnabled().unwrap()
        } else {
            true
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(handle) = self.handle.cast::<MUXC::Control>() {
            handle.SetIsEnabled(enabled).unwrap();
        }
    }

    pub fn preferred_size(&self) -> winio_primitive::Size {
        Size::from_native(self.handle.DesiredSize().unwrap())
    }

    pub fn loc(&self) -> Point {
        let left = MUXC::Canvas::GetLeft(&self.handle).unwrap();
        let top = MUXC::Canvas::GetTop(&self.handle).unwrap();
        Point::new(left, top)
    }

    pub fn set_loc(&mut self, p: Point) {
        MUXC::Canvas::SetLeft(&self.handle, p.x).unwrap();
        MUXC::Canvas::SetTop(&self.handle, p.y).unwrap();
    }

    pub fn size(&self) -> Size {
        let width = self.handle.Width().unwrap();
        let height = self.handle.Height().unwrap();
        Size::new(width, height)
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.SetWidth(v.width).unwrap();
        self.handle.SetHeight(v.height).unwrap();
    }
}
