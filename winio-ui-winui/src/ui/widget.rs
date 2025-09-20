use windows::core::Interface;
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{self as MUX, Controls as MUXC};

use crate::ui::Convertible;

#[derive(Debug)]
pub(crate) struct Widget {
    handle: MUX::FrameworkElement,
}

impl Widget {
    pub fn new(parent: impl AsContainer, handle: MUX::FrameworkElement) -> Self {
        handle
            .SetHorizontalAlignment(MUX::HorizontalAlignment::Center)
            .unwrap();
        handle
            .SetVerticalAlignment(MUX::VerticalAlignment::Center)
            .unwrap();
        let parent = parent.as_container();
        let canvas = parent.as_winui();
        canvas.Children().unwrap().Append(&handle).unwrap();
        canvas
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())
            .unwrap();
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

impl Drop for Widget {
    fn drop(&mut self) {
        if let Ok(parent) = self.handle.Parent() {
            if let Ok(parent) = parent.cast::<MUXC::Canvas>() {
                let children = parent.Children().unwrap();
                let mut index = 0;
                if children.IndexOf(&self.handle, &mut index).is_ok() {
                    children.RemoveAt(index as _).unwrap();
                }
            }
        }
    }
}

impl AsRawWidget for Widget {
    fn as_raw_widget(&self) -> RawWidget {
        RawWidget::WinUI(self.handle.clone())
    }
}
