use super::{Result, not_impl};

pub struct Runtime;

impl Runtime {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    #[cfg(not(windows))]
    pub fn set_app_id(&mut self, _app_id: &str) -> Result<()> {
        Ok(())
    }

    pub fn block_on<F: Future>(&self, _future: F) -> F::Output {
        not_impl()
    }
}
