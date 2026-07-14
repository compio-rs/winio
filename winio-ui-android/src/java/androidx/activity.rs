use crate::impl_listener;

jni::bind_java_type! {
    pub ActivityResultCaller => androidx.activity.result.ActivityResultCaller,
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract,
        ActivityResultLauncher => androidx.activity.result.ActivityResultLauncher,
        ActivityResultCallback => androidx.activity.result.ActivityResultCallback,
    },
    methods {
        fn register_for_activity_result(contract: &ActivityResultContract, callback: &ActivityResultCallback) -> ActivityResultLauncher,
    },
}

impl_listener!(ActivityResultCallback);

jni::bind_java_type! {
    pub ActivityResultContract => androidx.activity.result.contract.ActivityResultContract,
}

jni::bind_java_type! {
    pub GetContent => "androidx.activity.result.contract.ActivityResultContracts$GetContent",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    pub GetMultipleContents => "androidx.activity.result.contract.ActivityResultContracts$GetMultipleContents",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    pub OpenDocumentTree => "androidx.activity.result.contract.ActivityResultContracts$OpenDocumentTree",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    pub CreateDocument => "androidx.activity.result.contract.ActivityResultContracts$CreateDocument",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    pub ActivityResultLauncher => androidx.activity.result.ActivityResultLauncher,
    methods {
        fn launch(input: &JObject),
    },
}

jni::bind_java_type! {
    pub ActivityResultCallback => androidx.activity.result.ActivityResultCallback,
}
