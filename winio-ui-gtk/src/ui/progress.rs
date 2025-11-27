use std::{cell::Cell, rc::Rc, time::Duration};

use gtk4::glib::{ControlFlow, SourceId, object::Cast};
use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, ui::Widget};

#[derive(Debug)]
pub struct Progress {
    widget: gtk4::ProgressBar,
    handle: Widget,
    timer: Option<SourceId>,
    indeterminate: Rc<Cell<bool>>,
    min: usize,
    max: usize,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::ProgressBar::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let indeterminate = Rc::new(Cell::new(false));
        let timer = gtk4::glib::timeout_add_local(Duration::from_millis(100), {
            let widget = widget.clone();
            let indeterminate = indeterminate.clone();
            move || {
                if indeterminate.get() {
                    widget.pulse();
                }
                ControlFlow::Continue
            }
        });
        Ok(Self {
            widget,
            handle,
            timer: Some(timer),
            indeterminate,
            min: 0,
            max: 1,
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

    pub fn set_size(&mut self, mut s: Size) -> Result<()> {
        s.height = self.preferred_size()?.height;
        self.handle.set_size(s)
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.min)
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.max)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let pos = self.pos()?;
        self.min = v;
        self.set_pos(pos)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let pos = self.pos()?;
        self.max = v;
        self.set_pos(pos)
    }

    pub fn pos(&self) -> Result<usize> {
        Ok((self.widget.fraction() * ((self.max - self.min) as f64) + self.min as f64) as usize)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        self.widget
            .set_fraction(((pos - self.min) as f64) / ((self.max - self.min) as f64));
        Ok(())
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        Ok(self.indeterminate.get())
    }

    pub fn set_indeterminate(&mut self, v: bool) -> Result<()> {
        self.indeterminate.set(v);
        Ok(())
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.timer
            .take()
            .expect("Progress timer already taken")
            .remove();
    }
}

winio_handle::impl_as_widget!(Progress, handle);
