use crate::{Data, Result, not_impl, not_impl_fut};

#[derive(Debug)]
pub struct Clipboard;

impl Clipboard {
    pub fn new() -> Result<Self> {
        not_impl()
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        not_impl_fut().await
    }

    pub async fn write(&mut self, _data: &Data) -> Result<()> {
        not_impl_fut().await
    }

    pub fn clear(&mut self) -> Result<()> {
        not_impl()
    }

    pub async fn wait_change(&self) {
        not_impl_fut::<()>().await
    }
}
