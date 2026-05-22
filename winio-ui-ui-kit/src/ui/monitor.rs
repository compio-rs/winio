use objc2::{MainThreadMarker, rc::Retained};
use objc2_ui_kit::{UIApplication, UIWindowScene};
use winio_primitive::{Monitor, Point, Rect, Size};

use crate::{Error, Result, catch, ui::from_cgsize};

pub fn monitor_get_all() -> Result<Vec<Monitor>> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
    let mut res = vec![];
    catch(|| {
        let app = UIApplication::sharedApplication(mtm);
        for session in app.openSessions() {
            if let Some(scene) = session.scene()
                && let Ok(scene) = Retained::downcast::<UIWindowScene>(scene)
            {
                let screen = scene.screen();
                let bounds = screen.bounds();
                let frame_size = from_cgsize(bounds.size);
                let frame = Rect::new(Point::zero(), frame_size);

                let dpi = screen
                    .currentMode()
                    .map(|mode| from_cgsize(mode.size()))
                    .unwrap_or(Size::new(1.0, 1.0));

                let native_scale = screen.nativeScale();
                let dpi = Size::new(dpi.width / native_scale, dpi.height / native_scale);

                res.push(Monitor::new(frame, frame, dpi));
            }
        }
    })?;
    Ok(res)
}
