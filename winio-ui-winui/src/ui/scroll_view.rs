use inherit_methods_macro::inherit_methods;
use windows::core::Interface;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Rect, Size};
use winui3::Microsoft::UI::Xaml::{
    Controls as MUXC, FrameworkElement, HorizontalAlignment, VerticalAlignment,
};

use crate::{Result, Widget, ui::Convertible};

#[derive(Debug)]
pub struct ScrollView {
    handle: Widget,
    view: MUXC::ScrollViewer,
    canvas: MUXC::Canvas,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let view = MUXC::ScrollViewer::new()?;
        let canvas = MUXC::Canvas::new()?;
        canvas.SetHorizontalAlignment(HorizontalAlignment::Left)?;
        canvas.SetVerticalAlignment(VerticalAlignment::Top)?;
        view.SetContent(&canvas)?;
        view.SetHorizontalScrollBarVisibility(MUXC::ScrollBarVisibility::Auto)?;
        view.SetVerticalScrollBarVisibility(MUXC::ScrollBarVisibility::Auto)?;
        Ok(Self {
            handle: Widget::new(parent, view.cast()?)?,
            view,
            canvas,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        let rect = self
            .canvas
            .Children()?
            .into_iter()
            .map(|c| {
                let c = c.cast::<FrameworkElement>()?;
                let left = MUXC::Canvas::GetLeft(&c)?;
                let top = MUXC::Canvas::GetTop(&c)?;
                let size = Size::from_native(c.DesiredSize()?);
                Result::Ok(Rect::new(Point::new(left, top), size))
            })
            .reduce(|a, b| a.and_then(|a| b.map(|b| (a, b))).map(|(a, b)| a.union(&b)))
            .unwrap_or_else(|| Ok(Rect::zero()))?;
        self.canvas.SetWidth(rect.max_x())?;
        self.canvas.SetHeight(rect.max_y())?;
        Ok(())
    }

    pub fn hscroll(&self) -> Result<bool> {
        Ok(self.view.HorizontalScrollBarVisibility()? != MUXC::ScrollBarVisibility::Disabled)
    }

    pub fn set_hscroll(&mut self, v: bool) -> Result<()> {
        self.view.SetHorizontalScrollBarVisibility(if v {
            MUXC::ScrollBarVisibility::Auto
        } else {
            MUXC::ScrollBarVisibility::Disabled
        })?;
        Ok(())
    }

    pub fn vscroll(&self) -> Result<bool> {
        Ok(self.view.VerticalScrollBarVisibility()? != MUXC::ScrollBarVisibility::Disabled)
    }

    pub fn set_vscroll(&mut self, v: bool) -> Result<()> {
        self.view.SetVerticalScrollBarVisibility(if v {
            MUXC::ScrollBarVisibility::Auto
        } else {
            MUXC::ScrollBarVisibility::Disabled
        })?;
        Ok(())
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

winio_handle::impl_as_container!(ScrollView);
