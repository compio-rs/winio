use futures_util::Stream;
use winio_primitive::{DataEntry, DataFormat};

use crate::{Image, Result, not_impl, not_impl_any, not_impl_fut};

#[derive(Debug)]
pub struct Data;

impl Data {
    pub fn new() -> Result<Self> {
        not_impl()
    }

    pub fn is_empty(&self) -> bool {
        not_impl()
    }

    pub async fn insert(&mut self, _entry: DataEntry<Image>) -> Result<()> {
        not_impl_fut().await
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        Ok([not_impl_any()].into_iter())
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        futures_util::stream::once(not_impl_fut())
    }
}
