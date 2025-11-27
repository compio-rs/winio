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

use crate::{Error, Result, ui::Widget};

#[derive(Debug)]
pub struct Media {
    widget: gtk4::Video,
    handle: Widget,
    source: Option<gtk4::MediaFile>,
    image: gtk4::Widget,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::Video::new();
        widget.set_autoplay(false);
        let overlay = widget.first_child().ok_or(Error::NullPointer)?;
        let controls = overlay.last_child().ok_or(Error::NullPointer)?;
        controls.set_visible(false);
        let image = controls.prev_sibling().ok_or(Error::NullPointer)?;
        image.set_visible(false);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        Ok(Self {
            widget,
            handle,
            source: None,
            image,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::zero())
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn url(&self) -> Result<String> {
        Ok(self
            .source
            .as_ref()
            .and_then(|player| player.file())
            .map(|file| file.uri().to_string())
            .unwrap_or_default())
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
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
        let res = if let Some(err) = player.error() {
            Err(Error::Glib(err))
        } else {
            Ok(())
        };
        player.disconnect(cp);
        self.source = Some(player);
        res
    }

    pub fn play(&mut self) -> Result<()> {
        if let Some(player) = &self.source {
            player.play();
        }
        self.image.set_visible(false);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        if let Some(player) = &self.source {
            player.pause();
        }
        self.image.set_visible(false);
        Ok(())
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        Ok(self
            .source
            .as_ref()
            .map(|player| Duration::from_micros(player.duration() as _)))
    }

    pub fn current_time(&self) -> Result<Duration> {
        Ok(self
            .source
            .as_ref()
            .map(|player| Duration::from_micros(player.timestamp() as _))
            .unwrap_or_default())
    }

    pub fn set_current_time(&mut self, t: Duration) -> Result<()> {
        if let Some(player) = &self.source {
            player.seek(t.as_micros() as _);
            player.seek_success();
        }
        Ok(())
    }

    pub fn volume(&self) -> Result<f64> {
        Ok(self
            .source
            .as_ref()
            .map(|player| player.volume())
            .unwrap_or_default())
    }

    pub fn set_volume(&mut self, v: f64) -> Result<()> {
        if let Some(player) = &self.source {
            player.set_volume(v);
        }
        Ok(())
    }

    pub fn is_muted(&self) -> Result<bool> {
        Ok(self
            .source
            .as_ref()
            .map(|player| player.is_muted())
            .unwrap_or_default())
    }

    pub fn set_muted(&mut self, v: bool) -> Result<()> {
        if let Some(player) = &self.source {
            player.set_muted(v);
        }
        Ok(())
    }
}

winio_handle::impl_as_widget!(Media, handle);

impl Drop for Media {
    fn drop(&mut self) {
        self.widget.set_media_stream(None::<&gtk4::MediaStream>);
        self.source = None;
    }
}
