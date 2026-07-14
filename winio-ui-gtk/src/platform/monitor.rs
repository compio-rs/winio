use gtk4::{
    gdk::{
        self,
        prelude::{DisplayExt, MonitorExt},
    },
    glib::object::Cast,
};
use winio_primitive::{Monitor, Point, Rect, Size};

use crate::Result;

pub fn monitor_get_all() -> Result<Vec<Monitor>> {
    Ok(gdk::DisplayManager::get()
        .list_displays()
        .into_iter()
        .flat_map(|d| {
            d.monitors()
                .into_iter()
                .filter_map(|m| m.ok().and_then(|m| m.downcast::<gdk::Monitor>().ok()))
                .collect::<Vec<_>>()
        })
        .map(|m| {
            let geo = rect_from(m.geometry());
            let scale = m.scale();
            Monitor::new(geo * scale, geo * scale, Size::new(scale, scale))
        })
        .collect())
}

#[inline]
fn rect_from(r: gdk::Rectangle) -> Rect {
    Rect::new(
        Point::new(r.x() as _, r.y() as _),
        Size::new(r.width() as _, r.height() as _),
    )
}
