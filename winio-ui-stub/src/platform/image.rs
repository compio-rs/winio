use winio_primitive::Size;

use crate::{Result, not_impl};

#[derive(Debug)]
pub struct Image;

impl Image {
    pub fn size(&self) -> Result<Size> {
        not_impl()
    }

    pub fn set_size(&mut self, _size: Size) -> Result<()> {
        not_impl()
    }
}
