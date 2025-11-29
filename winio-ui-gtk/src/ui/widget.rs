use std::cell::Cell;

use gtk4::prelude::{Cast, FixedExt, WidgetExt};
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Size};

use crate::{Error, Result};

#[derive(Debug)]
pub(crate) struct Widget {
    widget: gtk4::Widget,
    preferred_size: Cell<Size>,
}

impl Widget {
    pub fn new(parent: impl AsContainer, widget: gtk4::Widget) -> Result<Self> {
        let parent = parent.as_container().to_gtk();
        parent.put(&widget, 0.0, 0.0);
        Ok(Self {
            widget,
            preferred_size: Cell::new(Size::new(f64::MAX, f64::MAX)),
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.widget.get_visible())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        self.widget.set_visible(v);
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.widget.get_sensitive())
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.widget.set_sensitive(v);
        Ok(())
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let (size, _) = self.widget.preferred_size();
        let mut preferred_size = self.preferred_size.get();
        preferred_size.width = preferred_size.width.min(size.width() as _);
        preferred_size.height = preferred_size.height.min(size.height() as _);
        self.preferred_size.set(preferred_size);
        Ok(preferred_size)
    }

    pub fn reset_preferred_size(&mut self) {
        self.preferred_size.set(Size::new(f64::MAX, f64::MAX));
    }

    pub fn loc(&self) -> Result<Point> {
        let parent = self.widget.parent().ok_or(Error::NullPointer)?;
        let fixed = parent
            .downcast::<gtk4::Fixed>()
            .map_err(|_| Error::CastFailed)?;
        let (x, y) = fixed.child_position(&self.widget);
        Ok(Point::new(x, y))
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        let parent = self.widget.parent().ok_or(Error::NullPointer)?;
        let fixed = parent
            .downcast::<gtk4::Fixed>()
            .map_err(|_| Error::CastFailed)?;
        fixed.move_(&self.widget, p.x, p.y);
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let preferred_size = self.preferred_size()?;
        Ok(Size::new(
            (self.widget.width() as f64).max(preferred_size.width),
            (self.widget.height() as f64).max(preferred_size.height),
        ))
    }

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.widget.set_size_request(s.width as _, s.height as _);
        Ok(())
    }

    pub fn tooltip(&self) -> Result<String> {
        Ok(self
            .widget
            .tooltip_text()
            .map(|s| s.to_string())
            .unwrap_or_default())
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        let s = if s.is_empty() { None } else { Some(s) };
        self.widget.set_tooltip_text(s);
        Ok(())
    }
}

impl AsRawWidget for Widget {
    fn as_raw_widget(&self) -> RawWidget {
        RawWidget::Gtk(self.widget.clone())
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        if let Some(parent) = self.widget.parent()
            && let Ok(fixed) = parent.downcast::<gtk4::Fixed>()
        {
            fixed.remove(&self.widget);
        }
    }
}
