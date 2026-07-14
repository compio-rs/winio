use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use jni::refs::Global;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{AView, BaseWidget, Context, FrameLayout, Result, current_activity, vm_exec};

jni::bind_java_type! {
    PlayerView => androidx.media3.ui.PlayerView,
    type_map {
        AView => android.view.View,
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
        view = AView,
        frame_layout = FrameLayout,
    }
}

jni::bind_java_type! {
    Player => androidx.media3.common.Player,
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

const REPEAT_MODE_OFF: i32 = 0;
const REPEAT_MODE_ONE: i32 = 1;

jni::bind_java_type! {
    ExoPlayer => androidx.media3.exoplayer.ExoPlayer,
    type_map {
        Player => androidx.media3.common.Player,
    },
    is_instance_of = {
        player = Player,
    }
}

jni::bind_java_type! {
    ExoPlayerBuilder => "androidx.media3.exoplayer.ExoPlayer$Builder",
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
    MediaItem => androidx.media3.common.MediaItem,
    methods {
        static fn from_uri(uri: &JString) -> MediaItem,
    }
}

jni::bind_java_type! {
    PlaybackParameters => androidx.media3.common.PlaybackParameters,
    fields {
        speed: jfloat,
    }
}

#[derive(Debug)]
pub struct Media {
    inner: BaseWidget<PlayerView<'static>>,
    player: Global<ExoPlayer<'static>>,
    uri: String,
}

#[inherit_methods(from = "self.inner")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = PlayerView::new(env, &act)?;
            let player = ExoPlayerBuilder::new(env, &act)?.build(env)?;
            widget.set_player(env, &player)?;
            widget.set_use_controller(env, false)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            let player = env.new_global_ref(player)?;
            Ok(Self {
                inner,
                player,
                uri: String::new(),
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn url(&self) -> Result<String> {
        Ok(self.uri.clone())
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| {
            let url = url.as_ref();
            self.uri = url.to_string();
            let url = env.new_string(url)?;
            let item = MediaItem::from_uri(env, &url)?;
            self.player.as_player().set_media_item(env, &item)?;
            self.player.as_player().prepare(env)?;
            self.player.as_player().play(env)?;
            Ok(())
        })
    }

    pub fn play(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.player.as_player().play(env)?;
            Ok(())
        })
    }

    pub fn pause(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.player.as_player().pause(env)?;
            Ok(())
        })
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        vm_exec(|env| {
            let dur = self.player.as_player().get_duration(env)?;
            if dur < 0 {
                Ok(None)
            } else {
                Ok(Some(Duration::from_millis(dur as u64)))
            }
        })
    }

    pub fn current_time(&self) -> Result<Duration> {
        vm_exec(|env| {
            let pos = self.player.as_player().get_current_position(env)?;
            Ok(Duration::from_millis(pos as u64))
        })
    }

    pub fn set_current_time(&mut self, time: Duration) -> Result<()> {
        vm_exec(|env| {
            self.player
                .as_player()
                .seek_to(env, time.as_millis() as i64)?;
            Ok(())
        })
    }

    pub fn volume(&self) -> Result<f64> {
        vm_exec(|env| {
            let v = self.player.as_player().get_volume(env)?;
            Ok(v as f64)
        })
    }

    pub fn set_volume(&mut self, v: f64) -> Result<()> {
        vm_exec(|env| {
            self.player.as_player().set_volume(env, v as f32)?;
            Ok(())
        })
    }

    pub fn is_muted(&self) -> Result<bool> {
        Ok(self.volume()? == 0.0)
    }

    pub fn set_muted(&mut self, v: bool) -> Result<()> {
        vm_exec(|env| {
            if v {
                self.player.as_player().mute(env)?;
            } else {
                self.player.as_player().unmute(env)?;
            }
            Ok(())
        })
    }

    pub fn is_looped(&self) -> Result<bool> {
        vm_exec(|env| {
            let mode = self.player.as_player().get_repeat_mode(env)?;
            Ok(mode != REPEAT_MODE_OFF)
        })
    }

    pub fn set_looped(&mut self, v: bool) -> Result<()> {
        vm_exec(|env| {
            let mode = if v { REPEAT_MODE_ONE } else { REPEAT_MODE_OFF };
            self.player.as_player().set_repeat_mode(env, mode)?;
            Ok(())
        })
    }

    pub fn playback_rate(&self) -> Result<f64> {
        vm_exec(|env| {
            let v = self
                .player
                .as_player()
                .get_playback_parameters(env)?
                .speed(env)?;
            Ok(v as f64)
        })
    }

    pub fn set_playback_rate(&mut self, v: f64) -> Result<()> {
        vm_exec(|env| {
            self.player.as_player().set_playback_speed(env, v as f32)?;
            Ok(())
        })
    }
}

winio_handle::impl_as_widget!(Media, inner);
