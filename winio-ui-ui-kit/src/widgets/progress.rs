use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_ui_kit::UIProgressView;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, catch, widgets::Widget};

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    view: Retained<UIProgressView>,
    minimum: usize,
    maximum: usize,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = UIProgressView::new(mtm);
            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;
            view.setProgress(0.0);

            Ok(Self {
                handle,
                view,
                minimum: 0,
                maximum: 10,
            })
        })
        .flatten()
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

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.minimum)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let pos = self.pos()?;
        self.minimum = v;
        self.maximum = self.maximum.max(v);
        self.set_pos(pos)
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.maximum)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let pos = self.pos()?;
        self.minimum = self.minimum.min(v);
        self.maximum = v.max(self.minimum);
        self.set_pos(pos)
    }

    pub fn pos(&self) -> Result<usize> {
        if self.maximum == self.minimum {
            return Ok(self.minimum);
        }
        catch(|| {
            (self.view.progress() * ((self.maximum - self.minimum) as f32) + self.minimum as f32)
                as usize
        })
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        let pos = pos.clamp(self.minimum, self.maximum);
        let progress = if self.maximum == self.minimum {
            0.0
        } else {
            (pos - self.minimum) as f32 / (self.maximum - self.minimum) as f32
        };
        catch(|| self.view.setProgress_animated(progress, true))
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        Ok(false)
    }

    pub fn set_indeterminate(&mut self, _v: bool) -> Result<()> {
        compio_log::error!("Indeterminate progress is not supported on iOS");
        Ok(())
    }
}

winio_handle::impl_as_widget!(Progress, handle);
