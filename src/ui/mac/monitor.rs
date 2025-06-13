use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{NSDeviceResolution, NSScreen};
use objc2_foundation::NSValue;

use crate::{
    Monitor, Point, Rect, Size,
    ui::{from_cgsize, transform_cgrect},
};

pub fn monitor_get_all() -> Vec<Monitor> {
    let mtm = MainThreadMarker::new().unwrap();
    let mut res = vec![];
    for screen in NSScreen::screens(mtm) {
        let frame = screen.frame();
        let vframe = screen.visibleFrame();

        let frame_size = from_cgsize(frame.size);
        let frame = transform_cgrect(frame_size, frame);
        let vframe = transform_cgrect(frame_size, vframe);

        let dpi = screen
            .deviceDescription()
            .objectForKey(unsafe { NSDeviceResolution })
            .map(|obj| from_cgsize(unsafe { Retained::cast_unchecked::<NSValue>(obj).sizeValue() }))
            .unwrap_or(Size::new(1.0, 1.0));

        res.push(Monitor::new(
            rect_scale(frame, dpi),
            rect_scale(vframe, dpi),
            dpi,
        ))
    }
    res
}

#[inline]
fn rect_scale(r: Rect, dpi: Size) -> Rect {
    Rect::new(
        Point::new(r.origin.x * dpi.width, r.origin.y * dpi.height),
        Size::new(r.size.width * dpi.width, r.size.height * dpi.height),
    )
}
