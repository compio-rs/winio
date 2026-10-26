use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::AdjustmentExt};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct UpDownEdit {
    on_change: Rc<Callback<()>>,
    widget: gtk4::SpinButton,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::SpinButton::with_range(0.0, 100.0, 1.0);
        widget.set_digits(0);
        widget.set_numeric(true);
        widget.set_wrap(false);
        widget.set_snap_to_ticks(false);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_change = Rc::new(Callback::new());
        widget.connect_value_changed({
            let on_change = on_change.clone();
            move |_| {
                on_change.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_change,
            widget,
            handle,
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
        Ok(self.widget.adjustment().lower() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        self.widget.adjustment().set_lower(v as _);
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.widget.adjustment().upper() as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        self.widget.adjustment().set_upper(v as _);
        Ok(())
    }

    pub fn step(&self) -> Result<usize> {
        Ok(self.widget.adjustment().step_increment() as _)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        self.widget.set_increments(v as _, 0.0);
        Ok(())
    }

    pub fn value(&self) -> Result<usize> {
        Ok(self.widget.value() as _)
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        self.widget.set_value(v as _);
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

winio_handle::impl_as_widget!(UpDownEdit, handle);
