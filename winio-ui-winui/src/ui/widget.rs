use windows::core::Interface;
use winio_handle::{AsRawWidget, AsWindow, RawWidget};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{self as MUX, Controls as MUXC};

use crate::ui::Convertible;

#[derive(Debug)]
pub(crate) struct Widget {
    handle: MUX::FrameworkElement,
}

impl Widget {
    pub fn new(parent: impl AsWindow, handle: MUX::FrameworkElement) -> Self {
        let parent = parent.as_window();
        let window = parent.as_winui();
        handle
            .SetHorizontalAlignment(MUX::HorizontalAlignment::Center)
            .unwrap();
        handle
            .SetVerticalAlignment(MUX::VerticalAlignment::Center)
            .unwrap();
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
        match self.handle.cast::<MUXC::Control>() { Ok(handle) => {
            handle.IsEnabled().unwrap()
        } _ => {
            true
        }}
    }

    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(handle) = self.handle.cast::<MUXC::Control>() {
            handle.SetIsEnabled(enabled).unwrap();
        }
    }

    pub fn preferred_size(&self) -> Size {
        Size::from_native(
            self.handle
                .MeasureOverride(Size::new(f64::INFINITY, f64::INFINITY).to_native())
                .unwrap(),
        )
    }

    pub fn min_size(&self) -> Size {
        let width = self.handle.MinWidth().unwrap();
        let height = self.handle.MinHeight().unwrap();
        Size::new(width, height)
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

impl AsRawWidget for Widget {
    fn as_raw_widget(&self) -> RawWidget {
        RawWidget::WinUI(self.handle.clone())
    }
}
