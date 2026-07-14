use super::super::util::DisplayMetrics;

jni::bind_java_type! {
    pub Resources => android.content.res.Resources,
    type_map {
        Configuration => android.content.res.Configuration,
        DisplayMetrics => android.util.DisplayMetrics,
        ResourcesTheme => "android.content.res.Resources$Theme",
    },
    methods {
        fn get_configuration() -> Configuration,
        fn get_display_metrics() -> DisplayMetrics,
        fn get_color(id: jint, theme: &ResourcesTheme) -> jint,
    },
}

jni::bind_java_type! {
    pub Configuration => android.content.res.Configuration,
    fields {
        pub ui_mode: jint,
    },
}

jni::bind_java_type! {
    pub ResourcesTheme => "android.content.res.Resources$Theme",
}
