use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn url(&self) -> Result<String> {
        not_impl()
    }

    pub async fn load(&mut self, _url: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn play(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn pause(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        not_impl()
    }

    pub fn current_time(&self) -> Result<Duration> {
        not_impl()
    }

    pub fn set_current_time(&mut self, _time: Duration) -> Result<()> {
        not_impl()
    }

    pub fn volume(&self) -> Result<f64> {
        not_impl()
    }

    pub fn set_volume(&mut self, _v: f64) -> Result<()> {
        not_impl()
    }

    pub fn is_muted(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_muted(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }
}

winio_handle::impl_as_widget!(Media, handle);
