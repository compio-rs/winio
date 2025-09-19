use inherit_methods_macro::inherit_methods;
use windows::core::Interface;
use winio_handle::{AsContainer, AsRawContainer, BorrowedContainer, RawContainer};
use winio_primitive::{Point, Rect, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, HorizontalAlignment, VerticalAlignment};

use crate::Widget;

#[derive(Debug)]
pub struct ScrollView {
    handle: Widget,
    view: MUXC::ScrollViewer,
    canvas: MUXC::Canvas,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Self {
        let view = MUXC::ScrollViewer::new().unwrap();
        let canvas = MUXC::Canvas::new().unwrap();
        canvas
            .SetHorizontalAlignment(HorizontalAlignment::Left)
            .unwrap();
        canvas.SetVerticalAlignment(VerticalAlignment::Top).unwrap();
        view.SetContent(&canvas).unwrap();
        view.SetHorizontalScrollBarVisibility(MUXC::ScrollBarVisibility::Auto)
            .unwrap();
        view.SetVerticalScrollBarVisibility(MUXC::ScrollBarVisibility::Auto)
            .unwrap();
        Self {
            handle: Widget::new(parent, view.cast().unwrap()),
            view,
            canvas,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        let rect = self
            .canvas
            .Children()
            .unwrap()
            .into_iter()
            .map(|c| {
                let left = MUXC::Canvas::GetLeft(&c).unwrap();
                let top = MUXC::Canvas::GetTop(&c).unwrap();
                let size = c.ActualSize().unwrap();
                Rect::new(Point::new(left, top), Size::new(size.X as _, size.Y as _))
            })
            .reduce(|a, b| a.union(&b))
            .unwrap_or_default();
        self.canvas.SetWidth(rect.max_x()).unwrap();
        self.canvas.SetHeight(rect.max_y()).unwrap();
    }

    pub fn hscroll(&self) -> bool {
        self.view.HorizontalScrollBarVisibility().unwrap() != MUXC::ScrollBarVisibility::Disabled
    }

    pub fn set_hscroll(&mut self, v: bool) {
        self.view
            .SetHorizontalScrollBarVisibility(if v {
                MUXC::ScrollBarVisibility::Auto
            } else {
                MUXC::ScrollBarVisibility::Disabled
            })
            .unwrap();
    }

    pub fn vscroll(&self) -> bool {
        self.view.VerticalScrollBarVisibility().unwrap() != MUXC::ScrollBarVisibility::Disabled
    }

    pub fn set_vscroll(&mut self, v: bool) {
        self.view
            .SetVerticalScrollBarVisibility(if v {
                MUXC::ScrollBarVisibility::Auto
            } else {
                MUXC::ScrollBarVisibility::Disabled
            })
            .unwrap();
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);

impl AsRawContainer for ScrollView {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::WinUI(self.canvas.clone())
    }
}

impl AsContainer for ScrollView {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}
