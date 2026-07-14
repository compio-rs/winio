jni::bind_java_type! {
    pub Layout => "android.R$layout",
    fields {
        static simple_list_item_activated_1 {
            sig = jint,
            name = "simple_list_item_activated_1",
        },
        static simple_spinner_item {
            sig = jint,
            name = "simple_spinner_item",
        },
        static simple_spinner_dropdown_item {
            sig = jint,
            name = "simple_spinner_dropdown_item",
        },
    }
}

jni::bind_java_type! {
    pub RColor => "android.R$color",
    fields {
        static system_accent1_500 {
            sig = jint,
            name = "system_accent1_500",
        },
    }
}
