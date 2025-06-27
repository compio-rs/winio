use crate::{Monitor, Point, Rect, Size, ui::QRect};

pub fn monitor_get_all() -> Vec<Monitor> {
    ffi::screen_all()
        .into_iter()
        .map(|m| {
            let dpi = Size::new(m.dpix / 96.0, m.dpiy / 96.0);
            Monitor::new(rect_from(m.geo, dpi), rect_from(m.avail_geo, dpi), dpi)
        })
        .collect()
}

#[inline]
fn rect_from(r: QRect, dpi: Size) -> Rect {
    Rect::new(
        Point::new(r.x1 as f64 * dpi.width, r.y1 as f64 * dpi.height),
        Size::new(
            (r.x2 - r.x1) as f64 * dpi.width,
            (r.y2 - r.y1) as f64 * dpi.height,
        ),
    )
}

#[cxx::bridge]
mod ffi {
    struct Monitor {
        geo: QRect,
        avail_geo: QRect,
        dpix: f64,
        dpiy: f64,
    }

    unsafe extern "C++-unwind" {
        include!("winio/src/ui/qt/monitor.hpp");

        type QRect = super::QRect;

        fn screen_all() -> Vec<Monitor>;
    }
}
