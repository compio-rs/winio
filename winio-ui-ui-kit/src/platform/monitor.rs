use objc2::{MainThreadMarker, rc::Retained};
use objc2_ui_kit::{UIApplication, UIWindowScene};
use winio_primitive::{Monitor, Point, Rect, Size};

use crate::{Error, Result, catch, from_cgsize};

pub fn monitor_get_all() -> Result<Vec<Monitor>> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
    let mut res = vec![];
    catch(|| {
        let app = UIApplication::sharedApplication(mtm);
        for scene in app.connectedScenes() {
            if let Ok(scene) = Retained::downcast::<UIWindowScene>(scene) {
                let screen = scene.screen();
                let bounds = screen.bounds();
                let frame_size = from_cgsize(bounds.size);
                let frame = Rect::new(Point::zero(), frame_size);

                let scale = screen.scale();
                let dpi = Size::new(scale, scale);

                res.push(Monitor::new(frame, frame, dpi));
            }
        }
    })?;
    Ok(res)
}
