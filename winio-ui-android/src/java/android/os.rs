jni::bind_java_type! {
    pub ParcelFileDescriptor => android.os.ParcelFileDescriptor,
    methods {
        fn get_fd() -> jint,
    }
}

jni::bind_java_type! {
    pub BuildVersion => "android.os.Build$VERSION",
    fields {
        #[allow(non_snake_case)]
        static SDK_INT: jint,
    },
}
