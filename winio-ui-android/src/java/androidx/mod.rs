use crate::{
    impl_listener,
    java::android::{content::Context, view::View, widget::FrameLayout},
};

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

jni::bind_java_type! {
    pub PlayerView => androidx.media3.ui.PlayerView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        FrameLayout => android.widget.FrameLayout,
        Player => androidx.media3.common.Player,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn set_use_controller(v: bool),
        fn set_player(player: &Player),
    },
    is_instance_of = {
        view = View,
        frame_layout = FrameLayout,
    }
}

jni::bind_java_type! {
    pub Player => androidx.media3.common.Player,
    type_map {
        MediaItem => androidx.media3.common.MediaItem,
        PlaybackParameters => androidx.media3.common.PlaybackParameters,
    },
    methods {
        fn play(),
        fn pause(),
        fn get_duration() -> jlong,
        fn get_current_position() -> jlong,
        fn seek_to(pos: jlong),
        fn get_volume() -> jfloat,
        fn set_volume(v: jfloat),
        fn is_playing() -> jboolean,
        fn set_playback_speed(v: jfloat),
        fn get_playback_parameters() -> PlaybackParameters,
        fn mute(),
        fn unmute(),
        fn prepare(),
        fn set_media_item(item: &MediaItem),
        fn set_repeat_mode(mode: jint),
        fn get_repeat_mode() -> jint,
    },
}

jni::bind_java_type! {
    pub ExoPlayer => androidx.media3.exoplayer.ExoPlayer,
    type_map {
        Player => androidx.media3.common.Player,
    },
    is_instance_of = {
        player = Player,
    }
}

jni::bind_java_type! {
    pub ExoPlayerBuilder => "androidx.media3.exoplayer.ExoPlayer$Builder",
    type_map {
        Context => android.content.Context,
        ExoPlayer => androidx.media3.exoplayer.ExoPlayer,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn build() -> ExoPlayer,
    },
}

jni::bind_java_type! {
    pub MediaItem => androidx.media3.common.MediaItem,
    methods {
        static fn from_uri(uri: &JString) -> MediaItem,
    }
}

jni::bind_java_type! {
    pub PlaybackParameters => androidx.media3.common.PlaybackParameters,
    fields {
        speed: jfloat,
    }
}

jni::bind_java_type! {
    pub ViewPager2 => androidx.viewpager2.widget.ViewPager2,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn set_adapter(adapter: &RecyclerViewAdapter),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
}
