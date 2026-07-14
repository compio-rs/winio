jni::bind_java_type! {
    pub ParcelFileDescriptor => android.os.ParcelFileDescriptor,
    methods {
        fn get_fd() -> jint,
    }
}
