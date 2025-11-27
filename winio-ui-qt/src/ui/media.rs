use std::{
    error::Error,
    fmt::{Debug, Display},
    time::Duration,
};

use compio_log::error;
use cxx::{ExternType, UniquePtr, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result,
    ui::{Widget, impl_static_cast},
};

pub struct Media {
    on_notify: Box<Callback<bool>>,
    widget: Widget<ffi::QVideoWidget>,
    player: UniquePtr<ffi::WinioMediaPlayer>,
}

#[inherit_methods(from = "self.widget")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_notify = Box::new(Callback::new());
        let widget = unsafe { ffi::new_video(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        let mut player = ffi::new_player()?;
        unsafe {
            ffi::player_connect_notify(
                player.pin_mut(),
                Self::on_notify,
                on_notify.as_ref() as *const _ as _,
            )?;
        }
        Ok(Self {
            on_notify,
            widget,
            player,
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
        Ok(self.player.source()?.try_into()?)
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
        self.player.pin_mut().setSource(&url.as_ref().try_into()?)?;
        unsafe {
            self.player
                .pin_mut()
                .setVideoOutput(self.widget.pin_mut().get_unchecked_mut())?;
        }
        if self.on_notify.wait().await {
            Ok(())
        } else {
            Err(self.player.error()?.into())
        }
    }

    pub fn play(&mut self) -> Result<()> {
        self.player.pin_mut().play()?;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.player.pin_mut().pause()?;
        Ok(())
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        let duration = self.player.duration()?.0;
        if duration == 0 {
            Ok(None)
        } else {
            Ok(Some(Duration::from_millis(duration as _)))
        }
    }

    pub fn current_time(&self) -> Result<Duration> {
        Ok(Duration::from_millis(self.player.position()?.0 as _))
    }

    pub fn set_current_time(&mut self, t: Duration) -> Result<()> {
        self.player
            .pin_mut()
            .setPosition(qint64(t.as_millis() as _))?;
        Ok(())
    }

    pub fn volume(&self) -> Result<f64> {
        Ok(self.player.volume()?)
    }

    pub fn set_volume(&mut self, v: f64) -> Result<()> {
        self.player.pin_mut().setVolume(v)?;
        Ok(())
    }

    pub fn is_muted(&self) -> Result<bool> {
        Ok(self.player.isMuted()?)
    }

    pub fn set_muted(&mut self, v: bool) -> Result<()> {
        self.player.pin_mut().setMuted(v)?;
        Ok(())
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
            if let Err(_e) = self.player.pin_mut().setVideoOutput(std::ptr::null_mut()) {
                error!("Failed to clear video output: {_e:?}");
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum QMediaPlayerError {
    NoError,
    ResourceError,
    FormatError,
    NetworkError,
    AccessDeniedError,
    ServiceMissingError,
}

unsafe impl ExternType for QMediaPlayerError {
    type Id = type_id!("QMediaPlayerError");
    type Kind = cxx::kind::Trivial;
}

impl Display for QMediaPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            QMediaPlayerError::NoError => "No error",
            QMediaPlayerError::ResourceError => "Resource error",
            QMediaPlayerError::FormatError => "Format error",
            QMediaPlayerError::NetworkError => "Network error",
            QMediaPlayerError::AccessDeniedError => "Access denied error",
            QMediaPlayerError::ServiceMissingError => "Service missing error",
        };
        f.write_str(s)
    }
}

impl Error for QMediaPlayerError {}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/media.hpp");

        type qint64 = super::qint64;
        type QUrl = crate::ui::QUrl;
        type QWidget = crate::ui::QWidget;
        type QMediaPlayerError = super::QMediaPlayerError;
        type QVideoWidget;
        type WinioMediaPlayer;

        unsafe fn new_video(parent: *mut QWidget) -> Result<UniquePtr<QVideoWidget>>;
        fn new_player() -> Result<UniquePtr<WinioMediaPlayer>>;

        fn play(self: Pin<&mut WinioMediaPlayer>) -> Result<()>;
        fn pause(self: Pin<&mut WinioMediaPlayer>) -> Result<()>;
        fn source(self: &WinioMediaPlayer) -> Result<QUrl>;
        fn setSource(self: Pin<&mut WinioMediaPlayer>, url: &QUrl) -> Result<()>;
        unsafe fn setVideoOutput(
            self: Pin<&mut WinioMediaPlayer>,
            w: *mut QVideoWidget,
        ) -> Result<()>;
        fn duration(self: &WinioMediaPlayer) -> Result<qint64>;
        fn position(self: &WinioMediaPlayer) -> Result<qint64>;
        fn setPosition(self: Pin<&mut WinioMediaPlayer>, p: qint64) -> Result<()>;
        fn volume(self: &WinioMediaPlayer) -> Result<f64>;
        fn setVolume(self: Pin<&mut WinioMediaPlayer>, v: f64) -> Result<()>;
        fn isMuted(self: &WinioMediaPlayer) -> Result<bool>;
        fn setMuted(self: Pin<&mut WinioMediaPlayer>, v: bool) -> Result<()>;
        fn error(self: &WinioMediaPlayer) -> Result<QMediaPlayerError>;

        unsafe fn player_connect_notify(
            p: Pin<&mut WinioMediaPlayer>,
            callback: unsafe fn(*const u8, bool),
            data: *const u8,
        ) -> Result<()>;
    }
}
