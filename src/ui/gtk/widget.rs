use gtk4::prelude::{Cast, FixedExt, GtkWindowExt, WidgetExt};

use crate::{AsRawWindow, AsWindow, Point, Size};

#[derive(Debug)]
pub struct Widget {
    widget: gtk4::Widget,
}

impl Widget {
    pub fn new(parent: impl AsWindow, widget: gtk4::Widget) -> Self {
        let parent = parent.as_window().as_raw_window();
        let swindow = parent.child().unwrap();
        let port = swindow.first_child().unwrap();
        let fixed = port.first_child().unwrap();
        widget.set_parent(&fixed);
        let fixed = fixed.downcast::<gtk4::Fixed>().unwrap();
        fixed.put(&widget, 0.0, 0.0);
        Self { widget }
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
        let (_, size) = self.widget.preferred_size();
        let (_, width, ..) = self
            .widget
            .measure(gtk4::Orientation::Horizontal, size.width());
        let (_, height, ..) = self
            .widget
            .measure(gtk4::Orientation::Vertical, size.height());
        Size::new(
            self.widget.width().max(width) as f64,
            self.widget.height().max(height) as f64,
        )
    }

    pub fn set_size(&self, s: Size) {
        self.widget.set_size_request(s.width as _, s.height as _)
    }
}
