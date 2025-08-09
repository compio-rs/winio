use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedWindow;
use winio_layout::{Enable, Layoutable, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A media player.
#[derive(Debug)]
pub struct Media {
    widget: sys::Media,
}

#[inherit_methods(from = "self.widget")]
impl Media {
    /// The current URL.
    pub fn url(&self) -> String;

    /// Set the current URL.
    pub fn set_url(&mut self, url: impl AsRef<str>);

    /// Load a media source.
    pub fn load(&mut self, url: impl AsRef<str>) {
        self.set_url(url);
    }

    /// Play.
    pub fn play(&mut self);

    /// Pause.
    pub fn pause(&mut self);

    /// Full duration of the media.
    pub fn full_time(&self) -> Option<Duration>;

    /// The current time.
    pub fn current_time(&self) -> Duration;

    /// Set the current time.
    pub fn set_current_time(&mut self, t: Duration);

    /// Seek to a new time.
    pub fn seek(&mut self, t: Duration) {
        self.set_current_time(t);
    }

    /// Volume.
    pub fn volume(&self) -> f64;

    /// Set the volume. The value should between 0.0 to 1.0.
    pub fn set_volume(&mut self, v: f64);

    /// If the player is muted.
    pub fn is_muted(&self) -> bool;

    /// Set if the player is muted.
    pub fn set_muted(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Visible for Media {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for Media {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Media {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);

    fn preferred_size(&self) -> Size;
}

/// Events of [`Media`].
#[non_exhaustive]
pub enum MediaEvent {}

impl Component for Media {
    type Event = MediaEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::Media::new(init);
        Self { widget }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(Media, widget);
