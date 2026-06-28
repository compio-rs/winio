use winio_primitive::{Margin, Point, Size};

use crate::{Result, current_activity, vm_exec};

jni::bind_java_type! {
    pub(crate) DisplayMetrics => android.util.DisplayMetrics,
    fields {
        density: float,
    }
}

pub(crate) fn display_density() -> Result<f32> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let resources = act.as_context().get_resources(env)?;
        let metrics = resources.get_display_metrics(env)?;
        Ok(metrics.density(env)?)
    })
}

pub(crate) fn logical_size(width: f32, height: f32) -> Result<Size> {
    let density = display_density()?;
    Ok(Size::new(
        (width / density) as f64,
        (height / density) as f64,
    ))
}

pub(crate) fn logical_point(x: f32, y: f32) -> Result<Point> {
    let density = display_density()?;
    Ok(Point::new((x / density) as f64, (y / density) as f64))
}

pub(crate) fn logical_margin(top: i32, right: i32, bottom: i32, left: i32) -> Result<Margin> {
    let density = display_density()?;
    Ok(Margin::new(
        (top as f32 / density) as f64,
        (right as f32 / density) as f64,
        (bottom as f32 / density) as f64,
        (left as f32 / density) as f64,
    ))
}

pub(crate) fn physical_size(size: Size) -> Result<(f32, f32)> {
    let density = display_density()?;
    Ok((size.width as f32 * density, size.height as f32 * density))
}

pub(crate) fn physical_point(point: Point) -> Result<(f32, f32)> {
    let density = display_density()?;
    Ok((point.x as f32 * density, point.y as f32 * density))
}
