use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::NSProgressIndicator;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, catch, ui::Widget};

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
    view: Retained<NSProgressIndicator>,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.mtm();

        catch(|| unsafe {
            let view = NSProgressIndicator::new(mtm);
            view.setIndeterminate(false);
            view.setUsesThreadedAnimation(false);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            Ok(Self { handle, view })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::new(0.0, 5.0))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        catch(|| self.view.minValue() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        catch(|| self.view.setMinValue(v as _))
    }

    pub fn maximum(&self) -> Result<usize> {
        catch(|| self.view.maxValue() as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        catch(|| self.view.setMaxValue(v as _))
    }

    pub fn pos(&self) -> Result<usize> {
        catch(|| self.view.doubleValue() as _)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        catch(|| self.view.setDoubleValue(pos as _))
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        catch(|| self.view.isIndeterminate())
    }

    pub fn set_indeterminate(&mut self, v: bool) -> Result<()> {
        catch(|| unsafe {
            self.view.setIndeterminate(v);
            if v {
                self.view.startAnimation(None);
            } else {
                self.view.stopAnimation(None);
            }
        })
    }
}

winio_handle::impl_as_widget!(Progress, handle);
