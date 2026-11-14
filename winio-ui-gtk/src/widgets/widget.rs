use std::cell::Cell;

use gtk4::prelude::{Cast, FixedExt, WidgetExt};
use winio_handle::{AsContainer, AsWidget, BorrowedWidget};
use winio_primitive::{Point, Size};

use crate::{Error, Result};

#[derive(Debug)]
pub(crate) struct Widget {
    widget: gtk4::Widget,
    preferred_size: Cell<Size>,
    min_size: Cell<Size>,
}

impl Widget {
    pub fn new(parent: impl AsContainer, widget: gtk4::Widget) -> Result<Self> {
        let parent = parent.as_container().to_gtk();
        parent.put(&widget, 0.0, 0.0);
        Ok(Self {
            widget,
            preferred_size: Cell::new(Size::new(f64::MAX, f64::MAX)),
            min_size: Cell::new(Size::new(f64::MAX, f64::MAX)),
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
        self.ensure_measured();
        Ok(self.preferred_size.get())
    }

    pub fn min_size(&self) -> Result<Size> {
        self.ensure_measured();
        Ok(self.min_size.get())
    }

    fn ensure_measured(&self) {
        if self.preferred_size.get().width != f64::MAX && self.min_size.get().width != f64::MAX {
            return;
        }

        let (request_width, request_height) = self.widget.size_request();
        let has_request = request_width >= 0 || request_height >= 0;
        if has_request {
            self.widget.set_size_request(-1, -1);
        }

        let (min_width, natural_width, ..) = self.widget.measure(gtk4::Orientation::Horizontal, -1);
        let (min_height, natural_height, ..) = self.widget.measure(gtk4::Orientation::Vertical, -1);

        if has_request {
            self.widget.set_size_request(request_width, request_height);
        }

        self.min_size
            .set(Size::new(min_width as _, min_height as _));
        self.preferred_size
            .set(Size::new(natural_width as _, natural_height as _));
    }

    pub fn reset_preferred_size(&mut self) {
        self.preferred_size.set(Size::new(f64::MAX, f64::MAX));
        self.min_size.set(Size::new(f64::MAX, f64::MAX));
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

impl AsWidget for Widget {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        BorrowedWidget::gtk(&self.widget)
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        self.widget.unparent();
    }
}
