use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows_core::Interface;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};
use winui3::Microsoft::UI::Xaml::Controls::{self as MUXC, Primitives::TickPlacement};

use crate::{GlobalRuntime, Result, Widget, widgets::Convertible};

#[derive(Debug)]
pub struct Slider {
    on_scroll: SendWrapper<Rc<Callback>>,
    handle: Widget,
    bar: MUXC::Slider,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let bar = MUXC::Slider::new()?;
        bar.SetTickPlacement(TickPlacement::Outside)?;
        bar.SetIsThumbToolTipEnabled(false)?;
        bar.SetMaximum(10.0)?;
        bar.SetMinimum(0.0)?;
        bar.SetValue(0.0)?;
        bar.SetTickFrequency(1.0)?;
        let on_scroll = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_scroll = on_scroll.clone();
            bar.ValueChanged(move |_, _| {
                on_scroll.signal::<GlobalRuntime>(());
                Ok(())
            })?
            .forget();
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

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn tick_pos(&self) -> Result<TickPosition> {
        let pl = match self.bar.TickPlacement()? {
            TickPlacement::None => TickPosition::None,
            TickPlacement::TopLeft => TickPosition::TopLeft,
            TickPlacement::BottomRight => TickPosition::BottomRight,
            _ => TickPosition::Both,
        };
        Ok(pl)
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()> {
        let v = match v {
            TickPosition::None => TickPlacement::None,
            TickPosition::TopLeft => TickPlacement::TopLeft,
            TickPosition::BottomRight => TickPlacement::BottomRight,
            TickPosition::Both => TickPlacement::Outside,
        };
        self.bar.SetTickPlacement(v)?;
        Ok(())
    }

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
        let max = self.maximum()?.max(v.saturating_add(1));
        self.apply_range(v, max)
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.bar.Maximum()? as usize)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let old_min = self.minimum()?;
        let min = if v < old_min.saturating_add(1) {
            v.saturating_sub(1)
        } else {
            old_min
        };
        let max = v.max(min.saturating_add(1));
        self.apply_range(min, max)
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(self.bar.TickFrequency()? as _)
    }

    pub fn set_freq(&mut self, v: usize) -> Result<()> {
        self.bar.SetTickFrequency(v.max(1) as _)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        Ok(self.bar.Value()? as _)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        self.bar.SetValue(v as _)?;
        Ok(())
    }

    fn apply_range(&mut self, min: usize, max: usize) -> Result<()> {
        let old_max = self.maximum()?;
        if min > old_max {
            self.bar.SetMaximum(max as _)?;
            self.bar.SetMinimum(min as _)?;
        } else {
            self.bar.SetMinimum(min as _)?;
            self.bar.SetMaximum(max as _)?;
        }
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_scroll.wait().await
    }
}

winio_handle::impl_as_widget!(Slider, handle);
