use crate::{Point, Rect, Size, ui::sys};

/// Represents the geometry of a monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    region: Rect,
    client: Rect,
    dpi: Size,
}

impl Monitor {
    pub(crate) fn new(region: Rect, client: Rect, dpi: Size) -> Self {
        Self {
            region,
            client,
            dpi,
        }
    }

    /// Retrieve all monitors.
    pub fn all() -> Vec<Self> {
        sys::monitor_get_all()
    }

    /// The physical region.
    pub fn region(&self) -> Rect {
        self.region
    }

    /// The client region.
    pub fn client(&self) -> Rect {
        self.client
    }

    /// Dpi of the monitor, 1.0 if no scale. You should take it into
    /// consideration when setting the location of windows.
    /// See [`Monitor::region_scaled`] & [`Monitor::client_scaled`].
    pub fn dpi(&self) -> Size {
        self.dpi
    }

    /// Scaled physical region.
    pub fn region_scaled(&self) -> Rect {
        div_rect(self.region, self.dpi)
    }

    /// Scaled client region.
    pub fn client_scaled(&self) -> Rect {
        div_rect(self.client, self.dpi)
    }
}

#[inline]
fn div_rect(r: Rect, s: Size) -> Rect {
    Rect::new(div_point(r.origin, s), div_size(r.size, s))
}

#[inline]
fn div_point(p: Point, s: Size) -> Point {
    Point::new(p.x / s.width, p.y / s.height)
}

#[inline]
fn div_size(s1: Size, s2: Size) -> Size {
    Size::new(s1.width / s2.width, s1.height / s2.height)
}
