use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use windows::{Foundation::Uri, Media::Core::MediaSource, core::Interface};
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, Widget};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    mpe: MUXC::MediaPlayerElement,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mpe = MUXC::MediaPlayerElement::new()?;
        Ok(Self {
            handle: Widget::new(parent, mpe.cast()?)?,
            mpe,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn url(&self) -> Result<String> {
        Ok(self
            .mpe
            .Source()
            .and_then(|source| source.cast::<MediaSource>())
            .and_then(|source| source.Uri())
            .and_then(|uri| uri.ToString())
            .map(|s| s.to_string_lossy())
            .unwrap_or_default())
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
        let url = percent_encoding::percent_decode_str(url.as_ref()).decode_utf8_lossy();
        let source = MediaSource::CreateFromUri(&Uri::CreateUri(&url.as_ref().into())?)?;
        self.mpe.SetSource(&source)?;
        source.OpenAsync()?.await
    }

    pub fn play(&mut self) -> Result<()> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.Play()?;
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.Pause()?;
        }
        Ok(())
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        Ok(self
            .mpe
            .MediaPlayer()
            .and_then(|player| player.NaturalDuration())
            .map(|d| d.into())
            .ok())
    }

    pub fn current_time(&self) -> Result<Duration> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            Ok(player.Position()?.into())
        } else {
            Ok(Duration::ZERO)
        }
    }

    pub fn set_current_time(&mut self, t: Duration) -> Result<()> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.SetPosition(t.into()).ok();
        }
        Ok(())
    }

    pub fn volume(&self) -> Result<f64> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            Ok(player.Volume()?)
        } else {
            Ok(0.0)
        }
    }

    pub fn set_volume(&mut self, v: f64) -> Result<()> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.SetVolume(v)?;
        }
        Ok(())
    }

    pub fn is_muted(&self) -> Result<bool> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            Ok(player.IsMuted()?)
        } else {
            Ok(false)
        }
    }

    pub fn set_muted(&mut self, v: bool) -> Result<()> {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.SetIsMuted(v)?;
        }
        Ok(())
    }
}

winio_handle::impl_as_widget!(Media, handle);

impl Drop for Media {
    fn drop(&mut self) {
        self.mpe.SetSource(None).ok();
    }
}
