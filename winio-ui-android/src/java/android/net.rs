use crate::java::lang::File;

jni::bind_java_type! {
    pub Uri => android.net.Uri,
    type_map {
        File => java.io.File,
    },
    methods {
        static fn parse(uri: JString) -> Uri,
        static fn from_file(file: &File) -> Uri,
        fn to_string() -> JString,
    },
}
