use winio_primitive::Size;

use crate::{sys, sys::Result};

/// An image that can be used as an icon of widgets.
///
/// The supported image formats are determined by the backend and are not
/// limited to ICO files.
pub struct Image(pub(crate) sys::Image);

impl Image {
    /// Size of the image.
    pub fn size(&self) -> Result<Size> {
        self.0.size()
    }

    /// Set the size of the image.
    pub fn set_size(&mut self, size: Size) -> Result<()> {
        self.0.set_size(size)
    }
}
