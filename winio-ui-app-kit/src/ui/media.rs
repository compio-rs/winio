use std::{ffi::c_void, ptr::null_mut, time::Duration};

use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
};
use objc2_av_foundation::{AVPlayer, AVPlayerItem, AVPlayerItemStatus};
use objc2_av_kit::{AVPlayerView, AVPlayerViewControlsStyle};
use objc2_core_media::CMTime;
use objc2_foundation::{
    NSDictionary, NSKeyValueChangeKey, NSKeyValueObservingOptions, NSObject,
    NSObjectNSKeyValueObserverRegistration, NSString, NSURL, ns_string,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Widget, from_nsstring};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    view: Retained<AVPlayerView>,
    url: Option<Retained<NSURL>>,
    delegate: Retained<PlayerDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = AVPlayerView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = PlayerDelegate::new(mtm);

            view.setControlsStyle(AVPlayerViewControlsStyle::None);

            Self {
                handle,
                view,
                url: None,
                delegate,
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

    pub async fn load(&mut self, url: impl AsRef<str>) -> bool {
        unsafe {
            self.url = NSURL::URLWithString(&NSString::from_str(url.as_ref()));
            if let Some(url) = &self.url {
                let mtm = MainThreadMarker::new().unwrap();
                let item = AVPlayerItem::playerItemWithURL(url, mtm);
                item.addObserver_forKeyPath_options_context(
                    &self.delegate,
                    ns_string!("status"),
                    NSKeyValueObservingOptions::New,
                    null_mut(),
                );
                self.view
                    .setPlayer(Some(&AVPlayer::playerWithPlayerItem(Some(&item), mtm)));
                self.delegate.ivars().notify.wait().await;
                item.status() == AVPlayerItemStatus::ReadyToPlay
            } else {
                false
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

    pub fn full_time(&self) -> Option<Duration> {
        unsafe {
            self.view
                .player()
                .and_then(|player| player.currentItem())
                .and_then(|item| {
                    let t = item.duration();
                    Duration::try_from_secs_f64(t.seconds()).ok()
                })
        }
    }

    pub fn current_time(&self) -> Duration {
        unsafe {
            self.view
                .player()
                .and_then(|player| {
                    let ct = player.currentTime();
                    Duration::try_from_secs_f64(ct.seconds()).ok()
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

#[derive(Debug, Default)]
struct PlayerDelegateIvars {
    notify: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioPlayerDelegate"]
    #[ivars = PlayerDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct PlayerDelegate;

    #[allow(non_snake_case)]
    impl PlayerDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(PlayerDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        unsafe fn observeValueForKeyPath_ofObject_change_context(
            &self,
            key_path: Option<&NSString>,
            _object: Option<&AnyObject>,
            _change: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
            _context: *mut c_void,
        ) {
            if let Some(path) = key_path{
                if path.isEqualToString(ns_string!("status")) {
                    self.ivars().notify.signal::<GlobalRuntime>(());
                }
            }
        }
    }
}

impl PlayerDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
