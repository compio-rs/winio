jni::bind_java_type! {
    pub DisplayMetrics => android.util.DisplayMetrics,
    fields {
        density: float,
    }
}

jni::bind_java_type! {
    pub SparseBooleanArray => android.util.SparseBooleanArray,
    methods {
        fn size() -> jint,
        fn key_at(index: jint) -> jint,
        fn value_at(index: jint) -> bool,
    }
}
