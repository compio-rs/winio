use std::{io, ops::Deref, time::Duration};

use compio::{compat::Adapter, runtime::Runtime};

use super::not_impl;

pub struct CompioAdapter {
    runtime: Runtime,
}

impl Deref for CompioAdapter {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl Adapter for CompioAdapter {
    fn new(runtime: Runtime) -> io::Result<Self> {
        Ok(Self { runtime })
    }

    async fn wait(&self, _timeout: Option<Duration>) -> io::Result<()> {
        not_impl()
    }

    fn clear(&self) -> io::Result<()> {
        Ok(())
    }
}
