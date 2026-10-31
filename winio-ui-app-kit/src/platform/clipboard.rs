use std::{cell::Cell, fmt::Debug, time::Duration};

use objc2::rc::Retained;
use objc2_app_kit::NSPasteboard;
use objc2_foundation::NSInteger;

use crate::{Result, Timer, catch, platform::Data};

/// The interval to poll the pasteboard change count.
///
/// `NSPasteboard` has no change notification, so the only way to observe
/// changes made by other applications is to poll it.
const CHANGE_INTERVAL: Duration = Duration::from_millis(200);

pub struct Clipboard {
    pasteboard: Retained<NSPasteboard>,
    timer: Timer,
    change_count: Cell<NSInteger>,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let (pasteboard, change_count) = catch(|| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let change_count = pasteboard.changeCount();
            (pasteboard, change_count)
        })?;
        let mut timer = Timer::new(CHANGE_INTERVAL)?;
        timer.start()?;
        Ok(Self {
            pasteboard,
            timer,
            change_count: Cell::new(change_count),
        })
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        Data::from_pasteboard(&self.pasteboard)
    }

    pub async fn write(&mut self, data: &Data) -> Result<()> {
        data.write_to(&self.pasteboard)?;
        self.sync()
    }

    pub fn clear(&mut self) -> Result<()> {
        catch(|| self.pasteboard.clearContents())?;
        self.sync()
    }

    pub async fn wait_change(&self) {
        loop {
            self.timer.wait().await;
            let change_count = self.pasteboard.changeCount();
            if change_count != self.change_count.get() {
                self.change_count.set(change_count);
                break;
            }
        }
    }

    fn sync(&self) -> Result<()> {
        let change_count = catch(|| self.pasteboard.changeCount())?;
        self.change_count.set(change_count);
        Ok(())
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = self.timer.stop();
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}
