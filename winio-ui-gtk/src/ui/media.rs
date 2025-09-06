use std::time::Duration;

use gtk4::{
    MediaFile,
    gio::prelude::FileExt,
    glib::object::Cast,
    prelude::{MediaFileExt, MediaStreamExt},
};
use inherit_methods_macro::inherit_methods;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Media {
    widget: gtk4::Video,
    handle: Widget,
    source: Option<gtk4::MediaFile>,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsWindow) -> Self {
        let widget = gtk4::Video::new();
        widget.set_autoplay(false);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Self {
            widget,
            handle,
            source: None,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn url(&self) -> String {
        self.source
            .as_ref()
            .and_then(|player| player.file())
            .map(|file| file.uri().to_string())
            .unwrap_or_default()
    }

    pub fn set_url(&mut self, url: impl AsRef<str>) {
        let player = MediaFile::for_file(&gtk4::gio::File::for_uri(url.as_ref()));
        self.widget.set_media_stream(Some(&player));
        self.source = Some(player);
    }

    pub fn play(&mut self) {
        if let Some(player) = &self.source {
            player.play();
        }
    }

    pub fn pause(&mut self) {
        if let Some(player) = &self.source {
            player.pause();
        }
    }

    pub fn full_time(&self) -> Option<Duration> {
        self.source
            .as_ref()
            .map(|player| Duration::from_micros(player.duration() as _))
    }

    pub fn current_time(&self) -> Duration {
        self.source
            .as_ref()
            .map(|player| Duration::from_micros(player.timestamp() as _))
            .unwrap_or_default()
    }

    pub fn set_current_time(&mut self, t: Duration) {
        if let Some(player) = &self.source {
            player.seek(t.as_micros() as _);
        }
    }

    pub fn volume(&self) -> f64 {
        self.source
            .as_ref()
            .map(|player| player.volume())
            .unwrap_or_default()
    }

    pub fn set_volume(&mut self, v: f64) {
        if let Some(player) = &self.source {
            player.set_volume(v);
        }
    }

    pub fn is_muted(&self) -> bool {
        self.source
            .as_ref()
            .map(|player| player.is_muted())
            .unwrap_or_default()
    }

    pub fn set_muted(&mut self, v: bool) {
        if let Some(player) = &self.source {
            player.set_muted(v);
        }
    }
}

winio_handle::impl_as_widget!(Media, handle);

impl Drop for Media {
    fn drop(&mut self) {
        self.widget.set_media_stream(None::<&gtk4::MediaStream>);
        self.source = None;
    }
}
