use crate::{Monitor, Point, Rect, Size, ui::QRect};

pub fn monitor_get_all() -> Vec<Monitor> {
    ffi::screen_all()
        .into_iter()
        .map(|m| {
            Monitor::new(
                rect_from(m.geo),
                rect_from(m.avail_geo),
                Size::new(1.0, 1.0),
            )
        })
        .collect()
}

#[inline]
fn rect_from(r: QRect) -> Rect {
    Rect::new(
        Point::new(r.x1 as _, r.y1 as _),
        Size::new((r.x2 - r.x1) as _, (r.y2 - r.y1) as _),
    )
}

#[cxx::bridge]
mod ffi {
    struct Monitor {
        geo: QRect,
        avail_geo: QRect,
    }

    unsafe extern "C++" {
        include!("winio/src/ui/qt/monitor.hpp");

        type QRect = super::QRect;

        fn screen_all() -> Vec<Monitor>;
    }
}
