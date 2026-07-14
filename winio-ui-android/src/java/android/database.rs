jni::bind_java_type! {
    pub Cursor => android.database.Cursor,
    methods {
        fn move_to_next() -> bool,
        fn get_string(column_index: jint) -> JString,
        fn get_column_index(column_name: JString) -> jint,
    }
}
