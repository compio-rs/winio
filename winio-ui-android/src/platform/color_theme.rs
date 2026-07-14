use winio_primitive::{Color, ColorTheme};

use crate::{Result, current_activity, java::android::r::RColor, vm_exec};

const UI_MODE_NIGHT_MASK: i32 = 0x30;
const UI_MODE_NIGHT_YES: i32 = 0x20;

pub fn color_theme() -> Result<ColorTheme> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let resources = act.as_context().get_resources(env)?;
        let config = resources.get_configuration(env)?;
        let ui_mode = config.ui_mode(env)?;
        Ok(if ui_mode & UI_MODE_NIGHT_MASK == UI_MODE_NIGHT_YES {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        })
    })
}

pub fn accent_color() -> Result<Color> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let resources = act.as_context().get_resources(env)?;
        let theme = act.as_context().get_theme(env)?;
        let color_id = RColor::system_accent1_500(env)?;
        let color_int = resources.get_color(env, color_id, theme)?;
        let a = ((color_int >> 24) & 0xFF) as u8;
        let r = ((color_int >> 16) & 0xFF) as u8;
        let g = ((color_int >> 8) & 0xFF) as u8;
        let b = (color_int & 0xFF) as u8;
        Ok(Color::new(r, g, b, a))
    })
}
