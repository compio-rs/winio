use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSTextAlignment, NSTextField};
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{Result, catch, ui::Widget};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
    view: Retained<NSTextField>,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();

        catch(|| unsafe {
            let view = NSTextField::new(parent.as_app_kit().mtm());
            view.setBezeled(false);
            view.setDrawsBackground(false);
            view.setEditable(false);
            view.setSelectable(false);
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
        let mut size = self.handle.preferred_size()?;
        size.width += 8.0;
        Ok(size)
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn halign(&self) -> Result<HAlign> {
        let align = catch(|| self.view.alignment())?;
        let align = match align {
            NSTextAlignment::Right => HAlign::Right,
            NSTextAlignment::Center => HAlign::Center,
            NSTextAlignment::Justified => HAlign::Stretch,
            _ => HAlign::Left,
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let align = match align {
            HAlign::Left => NSTextAlignment::Left,
            HAlign::Center => NSTextAlignment::Center,
            HAlign::Right => NSTextAlignment::Right,
            HAlign::Stretch => NSTextAlignment::Justified,
        };
        catch(|| self.view.setAlignment(align))
    }
}

winio_handle::impl_as_widget!(Label, handle);
