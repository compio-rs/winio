use std::{rc::Rc, time::Duration};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::NSTimer;
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::{Result, catch};

#[derive(Debug)]
pub struct Timer {
    inner: Option<Retained<NSTimer>>,
    interval: Duration,
    callback: Rc<Callback>,
}

impl Timer {
    pub fn new(interval: Duration) -> Result<Self> {
        Ok(Self {
            inner: None,
            interval,
            callback: Rc::new(Callback::new()),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.stop()?;
        let callback = self.callback.clone();
        let timer = catch(|| unsafe {
            let block = RcBlock::new(move |_| {
                callback.signal::<GlobalRuntime>(());
            });
            NSTimer::timerWithTimeInterval_repeats_block(self.interval.as_secs_f64(), true, &block)
        })?;
        self.inner = Some(timer);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(timer) = self.inner.take() {
            catch(|| timer.invalidate())?;
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.inner.is_some())
    }

    pub async fn wait(&self) {
        self.callback.wait().await
    }
}
