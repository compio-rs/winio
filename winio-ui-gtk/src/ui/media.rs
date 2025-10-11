use std::{rc::Rc, time::Duration};

use gtk4::{
    MediaFile,
    gio::prelude::FileExt,
    glib::object::{Cast, ObjectExt},
    prelude::{MediaFileExt, MediaStreamExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Media {
    widget: gtk4::Video,
    handle: Widget,
    source: Option<gtk4::MediaFile>,
    image: gtk4::Widget,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Self {
        let widget = gtk4::Video::new();
        widget.set_autoplay(false);
        let overlay = widget.first_child().unwrap();
        let controls = overlay.last_child().unwrap();
        controls.set_visible(false);
        let image = controls.prev_sibling().unwrap();
        image.set_visible(false);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Self {
            widget,
            handle,
            source: None,
            image,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn url(&self) -> String {
        self.source
            .as_ref()
            .and_then(|player| player.file())
            .map(|file| file.uri().to_string())
            .unwrap_or_default()
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> bool {
        let player = MediaFile::for_file(&gtk4::gio::File::for_uri(url.as_ref()));
        let callback = Rc::new(Callback::new());
        let cp = player.connect_prepared_notify({
            let callback = callback.clone();
            move |_| {
                callback.signal::<()>(());
            }
        });
        self.widget.set_media_stream(Some(&player));
        self.image.set_visible(false);
        callback.wait().await;
        let res = player.error().is_none();
        player.disconnect(cp);
        self.source = Some(player);
        res
    }

    pub fn play(&mut self) {
        if let Some(player) = &self.source {
            player.play();
        }
        self.image.set_visible(false);
    }

    pub fn pause(&mut self) {
        if let Some(player) = &self.source {
            player.pause();
        }
        self.image.set_visible(false);
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
            player.seek_success();
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
