use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A media player.
#[derive(Debug)]
pub struct Media {
    widget: sys::Media,
}

impl Failable for Media {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Media {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Media {
    /// The current URL.
    pub fn url(&self) -> Result<String>;

    /// Load a media source.
    ///
    /// Returns an error if the media source cannot be loaded.
    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
        self.widget.load(url).await
    }

    /// Play.
    pub fn play(&mut self) -> Result<()>;

    /// Pause.
    pub fn pause(&mut self) -> Result<()>;

    /// Full duration of the media.
    pub fn full_time(&self) -> Result<Option<Duration>>;

    /// The current time.
    pub fn current_time(&self) -> Result<Duration>;

    /// Set the current time.
    pub fn set_current_time(&mut self, t: Duration) -> Result<()>;

    /// Seek to a new time.
    pub fn seek(&mut self, t: Duration) -> Result<()> {
        self.set_current_time(t)
    }

    /// Volume.
    pub fn volume(&self) -> Result<f64>;

    /// Set the volume. The value should between 0.0 to 1.0.
    pub fn set_volume(&mut self, v: f64) -> Result<()>;

    /// If the player is muted.
    pub fn is_muted(&self) -> Result<bool>;

    /// Set if the player is muted.
    pub fn set_muted(&mut self, v: bool) -> Result<()>;

    /// If the player is looped.
    pub fn is_looped(&self) -> Result<bool>;

    /// Set if the player is looped.
    pub fn set_looped(&mut self, v: bool) -> Result<()>;

    /// Playback rate.
    pub fn playback_rate(&self) -> Result<f64>;

    /// Set playback rate.
    pub fn set_playback_rate(&mut self, v: f64) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Media {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Media {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Media {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

/// Events of [`Media`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MediaEvent {}

/// Messages of [`Media`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MediaMessage {
    /// No operation.
    Noop,
    /// Set the volume.
    SetVolume(f64),
    /// Set the muted state.
    SetMuted(bool),
    /// Set the looped state.
    SetLooped(bool),
    /// Set the current time.
    SetCurrentTime(Duration),
    /// Set the playback rate.
    SetPlaybackRate(f64),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
    /// Set the rect.
    SetRect(Rect),
}

impl Component for Media {
    type Error = Error;
    type Event = MediaEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = MediaMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Media::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        loop {
            std::future::pending::<()>().await
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MediaMessage::Noop => Ok(false),
            MediaMessage::SetVolume(volume) => {
                self.set_volume(volume)?;
                Ok(false)
            }
            MediaMessage::SetMuted(muted) => {
                self.set_muted(muted)?;
                Ok(false)
            }
            MediaMessage::SetLooped(looped) => {
                self.set_looped(looped)?;
                Ok(false)
            }
            MediaMessage::SetCurrentTime(current_time) => {
                self.set_current_time(current_time)?;
                Ok(false)
            }
            MediaMessage::SetPlaybackRate(playback_rate) => {
                self.set_playback_rate(playback_rate)?;
                Ok(false)
            }
            MediaMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            MediaMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            MediaMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
            MediaMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Media, widget);
