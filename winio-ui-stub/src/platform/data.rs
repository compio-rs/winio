use futures_util::Stream;
use winio_primitive::{DataEntry, DataFormat};

use crate::{Image, Result};

#[derive(Debug)]
pub struct Data;

impl Data {
    pub fn new() -> Result<Self> {
        not_impl()
    }

    pub fn is_empty(&self) -> bool {
        not_impl()
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        not_impl_fut().await
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        not_impl()
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        not_impl()
    }
}
