jni::bind_java_type! {
    pub Uri => android.net.Uri,
    methods {
        static fn parse(uri: JString) -> Uri,
        fn to_string() -> JString,
    },
}
