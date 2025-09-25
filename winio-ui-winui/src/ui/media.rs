use std::time::Duration;

use futures_util::StreamExt;
use inherit_methods_macro::inherit_methods;
use windows::{
    Foundation::{TypedEventHandler, Uri},
    Media::Core::{MediaSource, MediaSourceOpenOperationCompletedEventArgs},
    core::{Interface, Ref},
};
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::Widget;

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    mpe: MUXC::MediaPlayerElement,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Self {
        let mpe = MUXC::MediaPlayerElement::new().unwrap();
        Self {
            handle: Widget::new(parent, mpe.cast().unwrap()),
            mpe,
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

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn url(&self) -> String {
        self.mpe
            .Source()
            .and_then(|source| source.cast::<MediaSource>())
            .and_then(|source| source.Uri())
            .and_then(|uri| uri.ToString())
            .map(|s| s.to_string_lossy())
            .unwrap_or_default()
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> bool {
        let source =
            MediaSource::CreateFromUri(&Uri::CreateUri(&url.as_ref().into()).unwrap()).unwrap();
        let (tx, mut rx) = futures_channel::mpsc::unbounded();
        let token = source
            .OpenOperationCompleted(&TypedEventHandler::new(
                move |_, args: Ref<MediaSourceOpenOperationCompletedEventArgs>| {
                    if let Some(args) = args.as_ref() {
                        tx.unbounded_send(args.Error().is_err()).ok();
                    }
                    Ok(())
                },
            ))
            .unwrap();
        self.mpe.SetSource(&source).unwrap();
        let res = rx.next().await.unwrap_or_default();
        source.RemoveOpenOperationCompleted(token).unwrap();
        res
    }

    pub fn play(&mut self) {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.Play().unwrap();
        }
    }

    pub fn pause(&mut self) {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.Pause().unwrap();
        }
    }

    pub fn full_time(&self) -> Option<Duration> {
        self.mpe
            .MediaPlayer()
            .and_then(|player| player.NaturalDuration())
            .map(|d| d.into())
            .ok()
    }

    pub fn current_time(&self) -> Duration {
        self.mpe
            .MediaPlayer()
            .and_then(|player| player.Position())
            .map(|d| d.into())
            .unwrap_or_default()
    }

    pub fn set_current_time(&mut self, t: Duration) {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.SetPosition(t.into()).ok();
        }
    }

    pub fn volume(&self) -> f64 {
        self.mpe
            .MediaPlayer()
            .and_then(|player| player.Volume())
            .unwrap_or_default()
    }

    pub fn set_volume(&mut self, v: f64) {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.SetVolume(v).ok();
        }
    }

    pub fn is_muted(&self) -> bool {
        self.mpe
            .MediaPlayer()
            .and_then(|player| player.IsMuted())
            .unwrap_or_default()
    }

    pub fn set_muted(&mut self, v: bool) {
        if let Ok(player) = self.mpe.MediaPlayer() {
            player.SetIsMuted(v).ok();
        }
    }
}

winio_handle::impl_as_widget!(Media, handle);

impl Drop for Media {
    fn drop(&mut self) {
        self.mpe.SetSource(None).ok();
    }
}
