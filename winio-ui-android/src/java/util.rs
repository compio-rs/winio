jni::bind_java_type! {
    pub ArrayList => java.util.ArrayList,
    constructors {
        fn new(),
    },
    is_instance_of = {
        list = JList,
    },
}
