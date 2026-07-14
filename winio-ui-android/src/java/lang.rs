use crate::impl_listener;

jni::bind_java_type! {
    pub JCharSequence2 => java.lang.CharSequence,
    methods {
        fn to_string() -> JString,
    }
}

jni::bind_java_type! {
    pub JRunnable => java.lang.Runnable,
    methods {
        fn run(),
    }
}

impl_listener!(JRunnable);
