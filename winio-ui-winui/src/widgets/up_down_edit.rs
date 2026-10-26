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

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.number_box.Minimum()? as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        self.number_box.SetMinimum(v as _)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.number_box.Maximum()? as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        self.number_box.SetMaximum(v as _)?;
        Ok(())
    }

    pub fn step(&self) -> Result<usize> {
        Ok(self.number_box.SmallChange()? as _)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        self.number_box.SetSmallChange(v as _)?;
        Ok(())
    }

    pub fn value(&self) -> Result<usize> {
        self.number_box.Value().map(|v| v as _)
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        self.number_box.SetValue(v as _)?;
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

winio_handle::impl_as_widget!(UpDownEdit, handle);
