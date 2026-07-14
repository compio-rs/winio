use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use jni::refs::Global;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Result, current_activity,
    java::androidx::media3::{
        ExoPlayer, ExoPlayerBuilder, MediaItem, PlayerView,
        player::{REPEAT_MODE_OFF, REPEAT_MODE_ONE},
    },
    vm_exec,
};

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
