use crate::java::android::{content::Context, view::View, widget::FrameLayout};

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

pub mod player {
    pub const REPEAT_MODE_OFF: i32 = 0;
    pub const REPEAT_MODE_ONE: i32 = 1;
}
