use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows_core::Interface;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct UpDownEdit {
    on_change: SendWrapper<Rc<Callback>>,
    handle: Widget,
    number_box: MUXC::NumberBox,
}

#[inherit_methods(from = "self.handle")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let number_box = MUXC::NumberBox::new()?;
        number_box.SetAcceptsExpression(false)?;
        number_box.SetSpinButtonPlacementMode(MUXC::NumberBoxSpinButtonPlacementMode::Inline)?;
        number_box.SetMaximum(10.0)?;
        number_box.SetMinimum(0.0)?;
        number_box.SetValue(0.0)?;
        number_box.SetSmallChange(1.0)?;
        let on_change = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_change = on_change.clone();
            number_box
                .ValueChanged(move |_, _| {
                    on_change.signal::<GlobalRuntime>(());
                    Ok(())
                })?
                .forget();
        }
        let handle = Widget::new(parent, number_box.cast()?)?;
        Ok(Self {
            on_change,
            handle,
            number_box,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        let measured =
            self.number_box.ActualWidth()? > 0.0 || self.number_box.ActualHeight()? > 0.0;
        if !measured && (s.width <= 0.0 || s.height <= 0.0) {
            return Ok(());
        }
        self.handle.set_size(s)
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.number_box.Minimum()? as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?.max(v.saturating_add(1));
        self.apply_range(v, max)
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.number_box.Maximum()? as _)
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

    pub fn step(&self) -> Result<usize> {
        Ok(self.number_box.SmallChange()? as _)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        self.number_box.SetSmallChange(v.max(1) as _)?;
        Ok(())
    }

    pub fn value(&self) -> Result<usize> {
        self.number_box.Value().map(|v| v as _)
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        self.number_box.SetValue(v as _)?;
        Ok(())
    }

    fn apply_range(&mut self, min: usize, max: usize) -> Result<()> {
        let old_max = self.maximum()?;
        if min > old_max {
            self.number_box.SetMaximum(max as _)?;
            self.number_box.SetMinimum(min as _)?;
        } else {
            self.number_box.SetMinimum(min as _)?;
            self.number_box.SetMaximum(max as _)?;
        }
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

winio_handle::impl_as_widget!(UpDownEdit, handle);
