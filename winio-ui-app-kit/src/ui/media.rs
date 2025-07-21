use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadMarker, rc::Retained};
use objc2_av_foundation::AVPlayer;
use objc2_av_kit::{AVPlayerView, AVPlayerViewControlsStyle};
use objc2_core_media::CMTime;
use objc2_foundation::{NSString, NSURL};
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{Widget, from_nsstring};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    view: Retained<AVPlayerView>,
    url: Option<Retained<NSURL>>,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = AVPlayerView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            view.setControlsStyle(AVPlayerViewControlsStyle::None);

            Self {
                handle,
                view,
                url: None,
            }
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

    pub fn url(&self) -> String {
        self.url
            .as_ref()
            .and_then(|url| unsafe { url.absoluteString() })
            .map(|s| from_nsstring(&s))
            .unwrap_or_default()
    }

    pub fn set_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            self.url = NSURL::URLWithString(&NSString::from_str(url.as_ref()));
            if let Some(url) = &self.url {
                let mtm = MainThreadMarker::new().unwrap();
                self.view
                    .setPlayer(Some(&AVPlayer::playerWithURL(url, mtm)));
            }
        }
    }

    pub fn play(&mut self) {
        unsafe {
            if let Some(player) = self.view.player() {
                player.play();
            }
        }
    }

    pub fn pause(&mut self) {
        unsafe {
            if let Some(player) = self.view.player() {
                player.pause();
            }
        }
    }

    pub fn current_time(&self) -> Duration {
        unsafe {
            self.view
                .player()
                .map(|player| {
                    let ct = player.currentTime();
                    Duration::from_secs_f64(ct.seconds())
                })
                .unwrap_or_default()
        }
    }

    pub fn set_current_time(&mut self, t: Duration) {
        unsafe {
            if let Some(player) = self.view.player() {
                player.seekToTime(CMTime::with_seconds(t.as_secs_f64(), 600));
            }
        }
    }

    pub fn volume(&self) -> f64 {
        unsafe {
            self.view
                .player()
                .map(|player| player.volume() as _)
                .unwrap_or_default()
        }
    }

    pub fn set_volume(&mut self, v: f64) {
        unsafe {
            if let Some(player) = self.view.player() {
                player.setVolume(v as _);
            }
        }
    }

    pub fn is_muted(&self) -> bool {
        unsafe {
            self.view
                .player()
                .map(|player| player.isMuted())
                .unwrap_or_default()
        }
    }

    pub fn set_muted(&mut self, v: bool) {
        unsafe {
            if let Some(player) = self.view.player() {
                player.setMuted(v);
            }
        }
    }
}

winio_handle::impl_as_widget!(Media, handle);
