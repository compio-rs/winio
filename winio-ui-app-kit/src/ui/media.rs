use std::{ffi::c_void, ptr::null_mut, time::Duration};

use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
};
use objc2_av_foundation::{AVPlayerItem, AVPlayerItemStatus, AVPlayerLooper, AVQueuePlayer};
use objc2_av_kit::{AVPlayerView, AVPlayerViewControlsStyle};
use objc2_core_media::CMTime;
use objc2_foundation::{
    NSDictionary, NSKeyValueChangeKey, NSKeyValueObservingOptions, NSObject,
    NSObjectNSKeyValueObserverRegistration, NSString, NSURL, ns_string,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Error, GlobalRuntime, Result, Widget, catch, from_nsstring};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    view: Retained<AVPlayerView>,
    looper: Option<Retained<AVPlayerLooper>>,
    url: Option<Retained<NSURL>>,
    delegate: Retained<PlayerDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = AVPlayerView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = PlayerDelegate::new(mtm);

            view.setControlsStyle(AVPlayerViewControlsStyle::None);

            Ok(Self {
                handle,
                view,
                looper: None,
                url: None,
                delegate,
            })
        })
        .flatten()
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
            .url
            .as_ref()
            .and_then(|url| url.absoluteString())
            .map(|s| from_nsstring(&s))
            .unwrap_or_default())
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
        let item = catch(|| unsafe {
            let url = NSURL::URLWithString(&NSString::from_str(url.as_ref()))
                .ok_or(Error::NullPointer)?;
            let mtm = self.delegate.mtm();
            let item = AVPlayerItem::playerItemWithURL(&url, mtm);
            item.addObserver_forKeyPath_options_context(
                &self.delegate,
                ns_string!("status"),
                NSKeyValueObservingOptions::New,
                null_mut(),
            );
            self.view
                .setPlayer(Some(&AVQueuePlayer::playerWithPlayerItem(Some(&item), mtm)));
            self.url = Some(url);
            Ok(item)
        })
        .flatten()?;
        self.delegate.ivars().notify.wait().await;
        catch(|| unsafe {
            if item.status() == AVPlayerItemStatus::ReadyToPlay {
                Ok(())
            } else {
                Err(Error::NS(item.error()))
            }
        })
        .flatten()
    }

    pub fn play(&mut self) -> Result<()> {
        catch(|| unsafe {
            if let Some(player) = self.view.player() {
                player.play();
            }
        })
    }

    pub fn pause(&mut self) -> Result<()> {
        catch(|| unsafe {
            if let Some(player) = self.view.player() {
                player.pause();
            }
        })
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        catch(|| unsafe {
            self.view
                .player()
                .and_then(|player| player.currentItem())
                .and_then(|item| {
                    let t = item.duration();
                    Duration::try_from_secs_f64(t.seconds()).ok()
                })
        })
    }

    pub fn current_time(&self) -> Result<Duration> {
        catch(|| unsafe {
            self.view
                .player()
                .and_then(|player| {
                    let ct = player.currentTime();
                    Duration::try_from_secs_f64(ct.seconds()).ok()
                })
                .unwrap_or_default()
        })
    }

    pub fn set_current_time(&mut self, t: Duration) -> Result<()> {
        catch(|| unsafe {
            if let Some(player) = self.view.player() {
                player.seekToTime(CMTime::with_seconds(t.as_secs_f64(), 600));
            }
        })
    }

    pub fn volume(&self) -> Result<f64> {
        catch(|| unsafe {
            self.view
                .player()
                .map(|player| player.volume() as _)
                .unwrap_or_default()
        })
    }

    pub fn set_volume(&mut self, v: f64) -> Result<()> {
        catch(|| unsafe {
            if let Some(player) = self.view.player() {
                player.setVolume(v as _);
            }
        })
    }

    pub fn is_muted(&self) -> Result<bool> {
        catch(|| unsafe {
            self.view
                .player()
                .map(|player| player.isMuted())
                .unwrap_or_default()
        })
    }

    pub fn set_muted(&mut self, v: bool) -> Result<()> {
        catch(|| unsafe {
            if let Some(player) = self.view.player() {
                player.setMuted(v);
            }
        })
    }

    pub fn is_looped(&self) -> Result<bool> {
        Ok(self.looper.is_some())
    }

    pub fn set_looped(&mut self, v: bool) -> Result<()> {
        if v {
            if self.looper.is_none() {
                catch(|| unsafe {
                    if let Some(player) = self.view.player() {
                        let player = player
                            .downcast::<AVQueuePlayer>()
                            .map_err(|_| Error::NotSupported)?;
                        if let Some(item) = player.currentItem() {
                            let looper =
                                AVPlayerLooper::playerLooperWithPlayer_templateItem(&player, &item);
                            self.looper = Some(looper);
                        }
                    }
                    Ok(())
                })
                .flatten()?;
            }
        } else if let Some(looper) = self.looper.take() {
            catch(|| unsafe {
                looper.disableLooping();
            })?;
        }
        Ok(())
    }

    pub fn playback_rate(&self) -> Result<f64> {
        catch(|| unsafe {
            self.view
                .player()
                .map(|player| player.rate() as _)
                .unwrap_or_default()
        })
    }

    pub fn set_playback_rate(&mut self, v: f64) -> Result<()> {
        catch(|| unsafe {
            if let Some(player) = self.view.player() {
                player.setRate(v as _);
            }
        })
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
            if let Some(path) = key_path && path.isEqualToString(ns_string!("status")) {
                self.ivars().notify.signal::<GlobalRuntime>(());
            }
        }
    }
}

impl PlayerDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
