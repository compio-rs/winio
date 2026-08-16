use std::time::Duration;

use crate::{Result, not_impl};

#[derive(Debug)]
pub struct Timer {}

impl Timer {
    pub fn new(_duration: Duration) -> Result<Self> {
        not_impl()
    }

    pub fn start(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn stop(&mut self) -> Result<()> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool> {
        not_impl()
    }

    pub async fn wait(&self) {
        not_impl()
    }
}
