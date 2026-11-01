use crate::impl_listener;

jni::bind_java_type! {
    pub JCharSequence2 => java.lang.CharSequence,
    methods {
        fn to_string() -> JString,
    }
}

jni::bind_java_type! {
    pub File => java.io.File,
    constructors {
        fn new(path: JString),
    },
    methods {
        fn get_absolute_path() -> JString,
    },
}

jni::bind_java_type! {
    pub InputStream => java.io.InputStream,
    methods {
        fn read(buffer: jbyte[]) -> jint,
        fn close(),
    },
}

jni::bind_java_type! {
    pub JRunnable => java.lang.Runnable,
    methods {
        fn run(),
    }
}

impl_listener!(JRunnable);
