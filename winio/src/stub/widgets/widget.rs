use winio_handle::{
    AsContainer, AsWidget, AsWindow, BorrowedContainer, BorrowedWidget, BorrowedWindow,
};
use winio_primitive::{Point, Size};

use crate::stub::{Result, not_impl};

#[derive(Debug)]
pub(crate) struct Widget;

impl Widget {
    pub fn is_visible(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_visible(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_enabled(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn preferred_size(&self) -> Result<Size> {
        not_impl()
    }

    pub fn min_size(&self) -> Result<Size> {
        not_impl()
    }

    pub fn loc(&self) -> Result<Point> {
        not_impl()
    }

    pub fn set_loc(&mut self, _p: Point) -> Result<()> {
        not_impl()
    }

    pub fn size(&self) -> Result<Size> {
        not_impl()
    }

    pub fn set_size(&mut self, _v: Size) -> Result<()> {
        not_impl()
    }

    pub fn text(&self) -> Result<String> {
        not_impl()
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn tooltip(&self) -> Result<String> {
        not_impl()
    }

    pub fn set_tooltip(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }
}

impl AsWindow for Widget {
    fn as_window(&self) -> BorrowedWindow<'_> {
        not_impl()
    }
}

impl AsContainer for Widget {
    fn as_container(&self) -> BorrowedContainer<'_> {
        not_impl()
    }
}

impl AsWidget for Widget {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        not_impl()
    }
}
