use std::{fmt::Debug, time::Duration};

use cxx::{ExternType, UniquePtr, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, impl_static_cast},
};

pub struct Media {
    on_notify: Box<Callback<bool>>,
    widget: Widget<ffi::QVideoWidget>,
    player: UniquePtr<ffi::WinioMediaPlayer>,
}

#[inherit_methods(from = "self.widget")]
impl Media {
    pub fn new(parent: impl AsWindow) -> Self {
        let on_notify = Box::new(Callback::new());
        let widget = unsafe { ffi::new_video(parent.as_window().as_qt()) };
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        let mut player = ffi::new_player();
        unsafe {
            ffi::player_connect_notify(
                player.pin_mut(),
                Self::on_notify,
                on_notify.as_ref() as *const _ as _,
            );
        }
        Self {
            on_notify,
            widget,
            player,
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

    pub fn url(&self) -> String {
        self.player.source().into()
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> bool {
        self.player.pin_mut().setSource(&url.as_ref().into());
        unsafe {
            self.player
                .pin_mut()
                .setVideoOutput(self.widget.pin_mut().get_unchecked_mut());
        }
        self.on_notify.wait().await
    }

    pub fn play(&mut self) {
        self.player.pin_mut().play();
    }

    pub fn pause(&mut self) {
        self.player.pin_mut().pause();
    }

    pub fn full_time(&self) -> Option<Duration> {
        let duration = self.player.duration().0;
        if duration == 0 {
            None
        } else {
            Some(Duration::from_millis(duration as _))
        }
    }

    pub fn current_time(&self) -> Duration {
        Duration::from_millis(self.player.position().0 as _)
    }

    pub fn set_current_time(&mut self, t: Duration) {
        self.player
            .pin_mut()
            .setPosition(qint64(t.as_millis() as _));
    }

    pub fn volume(&self) -> f64 {
        self.player.volume()
    }

    pub fn set_volume(&mut self, v: f64) {
        self.player.pin_mut().setVolume(v);
    }

    pub fn is_muted(&self) -> bool {
        self.player.isMuted()
    }

    pub fn set_muted(&mut self, v: bool) {
        self.player.pin_mut().setMuted(v);
    }

    fn on_notify(c: *const u8, loaded: bool) {
        let c = c as *const Callback<bool>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(loaded);
        }
    }
}

impl Debug for Media {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Media")
            .field("widget", &self.widget)
            .finish_non_exhaustive()
    }
}

winio_handle::impl_as_widget!(Media, widget);

impl Drop for Media {
    fn drop(&mut self) {
        unsafe {
            self.player.pin_mut().setVideoOutput(std::ptr::null_mut());
        }
    }
}

impl_static_cast!(ffi::QVideoWidget, ffi::QWidget);

#[allow(non_camel_case_types)]
struct qint64(i64);

unsafe impl ExternType for qint64 {
    type Id = type_id!("qint64");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/media.hpp");

        type qint64 = super::qint64;
        type QUrl = crate::ui::QUrl;
        type QWidget = crate::ui::QWidget;
        type QVideoWidget;
        type WinioMediaPlayer;

        unsafe fn new_video(parent: *mut QWidget) -> UniquePtr<QVideoWidget>;
        fn new_player() -> UniquePtr<WinioMediaPlayer>;

        fn play(self: Pin<&mut WinioMediaPlayer>);
        fn pause(self: Pin<&mut WinioMediaPlayer>);
        fn source(self: &WinioMediaPlayer) -> QUrl;
        fn setSource(self: Pin<&mut WinioMediaPlayer>, url: &QUrl);
        unsafe fn setVideoOutput(self: Pin<&mut WinioMediaPlayer>, w: *mut QVideoWidget);
        fn duration(self: &WinioMediaPlayer) -> qint64;
        fn position(self: &WinioMediaPlayer) -> qint64;
        fn setPosition(self: Pin<&mut WinioMediaPlayer>, p: qint64);
        fn volume(self: &WinioMediaPlayer) -> f64;
        fn setVolume(self: Pin<&mut WinioMediaPlayer>, v: f64);
        fn isMuted(self: &WinioMediaPlayer) -> bool;
        fn setMuted(self: Pin<&mut WinioMediaPlayer>, v: bool);

        unsafe fn player_connect_notify(
            p: Pin<&mut WinioMediaPlayer>,
            callback: unsafe fn(*const u8, bool),
            data: *const u8,
        );
    }
}
