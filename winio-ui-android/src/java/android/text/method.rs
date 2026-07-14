jni::bind_java_type! {
    pub MovementMethod => android.text.method.MovementMethod,
}

jni::bind_java_type! {
    pub LinkMovementMethod => android.text.method.LinkMovementMethod,
    type_map {
        MovementMethod => android.text.method.MovementMethod,
    },
    methods {
        static fn get_instance() -> MovementMethod,
    },
    is_instance_of = {
        base = MovementMethod,
    }
}
