use std::cell::Cell;

use gtk4::prelude::{Cast, FixedExt, GtkWindowExt, WidgetExt};

use crate::{AsRawWindow, AsWindow, Point, Size};

#[derive(Debug)]
pub struct Widget {
    widget: gtk4::Widget,
    preferred_size: Cell<Size>,
}

impl Widget {
    pub fn new(parent: impl AsWindow, widget: gtk4::Widget) -> Self {
        let parent = parent.as_window().as_raw_window();
        let swindow = parent.child().unwrap();
        let port = swindow.first_child().unwrap();
        let fixed = port.first_child().unwrap();
        let fixed = fixed.downcast::<gtk4::Fixed>().unwrap();
        fixed.put(&widget, 0.0, 0.0);
        Self {
            widget,
            preferred_size: Cell::new(Size::new(f64::MAX, f64::MAX)),
        }
    }

    pub fn preferred_size(&self) -> Size {
        let (size, _) = self.widget.preferred_size();
        let mut preferred_size = self.preferred_size.get();
        preferred_size.width = preferred_size.width.min(size.width() as _);
        preferred_size.height = preferred_size.height.min(size.height() as _);
        self.preferred_size.set(preferred_size);
        preferred_size
    }

    pub fn reset_preferred_size(&mut self) {
        self.preferred_size.set(Size::new(f64::MAX, f64::MAX));
    }

    pub fn loc(&self) -> Point {
        let parent = self.widget.parent().unwrap();
        let fixed = parent.downcast::<gtk4::Fixed>().unwrap();
        let (x, y) = fixed.child_position(&self.widget);
        Point::new(x, y)
    }

    pub fn set_loc(&self, p: Point) {
        let parent = self.widget.parent().unwrap();
        let fixed = parent.downcast::<gtk4::Fixed>().unwrap();
        fixed.move_(&self.widget, p.x, p.y);
    }

    pub fn size(&self) -> Size {
        let preferred_size = self.preferred_size();
        Size::new(
            (self.widget.width() as f64).max(preferred_size.width),
            (self.widget.height() as f64).max(preferred_size.height),
        )
    }

    pub fn set_size(&self, s: Size) {
        self.widget.set_size_request(s.width as _, s.height as _)
    }
}
