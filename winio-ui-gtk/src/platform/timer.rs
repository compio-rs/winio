use std::{rc::Rc, time::Duration};

use gtk4::glib::JoinHandle;
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::Result;

#[derive(Debug)]
pub struct Timer {
    interval: Duration,
    callback: Rc<Callback>,
    handle: Option<JoinHandle<()>>,
}

impl Timer {
    pub fn new(interval: Duration) -> Result<Self> {
        Ok(Self {
            interval,
            callback: Rc::new(Callback::new()),
            handle: None,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let callback = self.callback.clone();
        let interval = self.interval;
        let handle = gtk4::glib::spawn_future_local(async move {
            loop {
                callback.signal::<GlobalRuntime>(());
                gtk4::glib::timeout_future(interval).await;
            }
        });
        self.handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.handle.is_some())
    }

    pub async fn wait(&self) {
        self.callback.wait().await
    }
}
