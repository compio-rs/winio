use image::DynamicImage;

use crate::{Result, not_impl};

#[derive(Debug)]
pub struct Image;

impl Image {
    pub fn new(_image: DynamicImage) -> Result<Self> {
        not_impl()
    }
}
