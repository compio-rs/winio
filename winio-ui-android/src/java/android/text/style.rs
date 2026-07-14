jni::bind_java_type! {
    pub ClickableSpan => android.text.style.ClickableSpan,
}

jni::bind_java_type! {
    pub URLSpan => android.text.style.URLSpan,
    type_map {
        ClickableSpan => android.text.style.ClickableSpan,
    },
    constructors {
        fn new(url: &JString),
    },
    methods {
        fn get_u_r_l() -> JString,
    },
    is_instance_of = {
        base = ClickableSpan,
    }
}
