use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A media player.
#[derive(Debug)]
pub struct Media {
    widget: sys::Media,
    volume_prop: Child<PropSink<f64>>,
    muted_prop: Child<PropSink<bool>>,
    looped_prop: Child<PropSink<bool>>,
    current_time_prop: Child<PropSink<Duration>>,
    playback_rate_prop: Child<PropSink<f64>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
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

    /// Property for [`Media::volume`].
    pub fn volume_prop(&self) -> &PropSink<f64> {
        &self.volume_prop
    }

    /// Property for [`Media::is_muted`].
    pub fn muted_prop(&self) -> &PropSink<bool> {
        &self.muted_prop
    }

    /// Property for [`Media::is_looped`].
    pub fn looped_prop(&self) -> &PropSink<bool> {
        &self.looped_prop
    }

    /// Property for [`Media::current_time`].
    pub fn current_time_prop(&self) -> &PropSink<Duration> {
        &self.current_time_prop
    }

    /// Property for [`Media::playback_rate`].
    pub fn playback_rate_prop(&self) -> &PropSink<f64> {
        &self.playback_rate_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
    }
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

    fn set_size(&mut self, v: Size) -> Result<()>;
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
    /// The volume has been changed.
    ChangeVolume,
    /// The muted state has been changed.
    ChangeMuted,
    /// The looped state has been changed.
    ChangeLooped,
    /// The current time has been changed.
    ChangeCurrentTime,
    /// The playback rate has been changed.
    ChangePlaybackRate,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for Media {
    type Error = Error;
    type Event = MediaEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = MediaMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Media::new(init)?;
        let Ok(volume_prop) = Child::<PropSink<f64>>::init(1.0).await;
        let Ok(muted_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(looped_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(current_time_prop) = Child::<PropSink<Duration>>::init(Duration::ZERO).await;
        let Ok(playback_rate_prop) = Child::<PropSink<f64>>::init(1.0).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            volume_prop,
            muted_prop,
            looped_prop,
            current_time_prop,
            playback_rate_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MediaMessage::Noop,
            self.volume_prop => { PropSinkEvent::Changed => MediaMessage::ChangeVolume },
            self.muted_prop => { PropSinkEvent::Changed => MediaMessage::ChangeMuted },
            self.looped_prop => { PropSinkEvent::Changed => MediaMessage::ChangeLooped },
            self.current_time_prop => { PropSinkEvent::Changed => MediaMessage::ChangeCurrentTime },
            self.playback_rate_prop => { PropSinkEvent::Changed => MediaMessage::ChangePlaybackRate },
            self.enabled_prop => { PropSinkEvent::Changed => MediaMessage::ChangeEnabled },
            self.visible_prop => { PropSinkEvent::Changed => MediaMessage::ChangeVisible },
            self.tooltip_prop => { PropSinkEvent::Changed => MediaMessage::ChangeTooltip },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.volume_prop.update().await;
        let Ok(r1) = self.muted_prop.update().await;
        let Ok(r2) = self.looped_prop.update().await;
        let Ok(r3) = self.current_time_prop.update().await;
        let Ok(r4) = self.playback_rate_prop.update().await;
        let Ok(r5) = self.enabled_prop.update().await;
        let Ok(r6) = self.visible_prop.update().await;
        let Ok(r7) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5 || r6 || r7)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MediaMessage::Noop => Ok(false),
            MediaMessage::ChangeVolume => {
                self.widget.set_volume(**self.volume_prop)?;
                Ok(true)
            }
            MediaMessage::ChangeMuted => {
                self.widget.set_muted(**self.muted_prop)?;
                Ok(true)
            }
            MediaMessage::ChangeLooped => {
                self.widget.set_looped(**self.looped_prop)?;
                Ok(true)
            }
            MediaMessage::ChangeCurrentTime => {
                self.widget.set_current_time(**self.current_time_prop)?;
                Ok(true)
            }
            MediaMessage::ChangePlaybackRate => {
                self.widget.set_playback_rate(**self.playback_rate_prop)?;
                Ok(true)
            }
            MediaMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            MediaMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            MediaMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Media, widget);
