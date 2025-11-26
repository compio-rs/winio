use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::Interface;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size};
use winui3::Microsoft::UI::Xaml::Controls::{
    self as MUXC,
    Primitives::{ScrollEventHandler, ScrollingIndicatorMode},
};

use crate::{GlobalRuntime, Result, Widget, ui::Convertible};

#[derive(Debug)]
pub struct ScrollBar {
    on_scroll: SendWrapper<Rc<Callback>>,
    handle: Widget,
    bar: MUXC::Primitives::ScrollBar,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let bar = MUXC::Primitives::ScrollBar::new()?;
        bar.SetIndicatorMode(ScrollingIndicatorMode::MouseIndicator)?;
        let on_scroll = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_scroll = on_scroll.clone();
            bar.Scroll(&ScrollEventHandler::new(move |_, _| {
                on_scroll.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        let handle = Widget::new(parent, bar.cast()?)?;
        Ok(Self {
            on_scroll,
            handle,
            bar,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let size = self.handle.preferred_size()?;
        let size = match self.orient()? {
            Orient::Horizontal => Size::new(0.0, size.height),
            Orient::Vertical => Size::new(size.width, 0.0),
        };
        Ok(size)
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn orient(&self) -> Result<Orient> {
        Ok(Orient::from_native(self.bar.Orientation()?))
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        self.bar.SetOrientation(v.to_native())?;
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.bar.Minimum()? as usize)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        self.bar.SetMinimum(v as _)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.bar.Maximum()? as usize + self.page()?)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let page = self.page()?;
        self.bar.SetMaximum(v.saturating_sub(page) as _)?;
        Ok(())
    }

    pub fn page(&self) -> Result<usize> {
        Ok(self.bar.ViewportSize()? as _)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        self.bar.SetViewportSize(v as _)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        Ok(self.bar.Value()? as _)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        self.bar.SetValue(v as _)?;
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_scroll.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
