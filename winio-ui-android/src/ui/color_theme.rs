use winio_primitive::ColorTheme;

use crate::{current_activity, vm_exec};

jni::bind_java_type! {
    pub(crate) Resources => android.content.res.Resources,
    type_map {
        Configuration => android.content.res.Configuration,
    },
    methods {
        fn get_configuration() -> Configuration,
    },
}

jni::bind_java_type! {
    Configuration => android.content.res.Configuration,
    fields {
        pub ui_mode: jint,
    },
}

const UI_MODE_NIGHT_MASK: i32 = 0x30;
const UI_MODE_NIGHT_YES: i32 = 0x20;

pub fn color_theme() -> crate::Result<ColorTheme> {
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
