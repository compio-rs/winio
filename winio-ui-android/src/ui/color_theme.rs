use winio_primitive::ColorTheme;

use crate::{current_activity, vm_exec};

const UI_MODE_NIGHT_MASK: i32 = 0x30;
const UI_MODE_NIGHT_YES: i32 = 0x20;

pub fn color_theme() -> crate::Result<ColorTheme> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let resources = env
            .call_method(
                act,
                jni::jni_str!("getResources"),
                jni::jni_sig!("()Landroid/content/res/Resources;"),
                &[],
            )?
            .l()?;
        let config = env
            .call_method(
                resources,
                jni::jni_str!("getConfiguration"),
                jni::jni_sig!("()Landroid/content/res/Configuration;"),
                &[],
            )?
            .l()?;
        let ui_mode = env
            .get_field(config, jni::jni_str!("uiMode"), jni::jni_sig!("I"))?
            .i()?;
        Ok(if ui_mode & UI_MODE_NIGHT_MASK == UI_MODE_NIGHT_YES {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        })
    })
}
