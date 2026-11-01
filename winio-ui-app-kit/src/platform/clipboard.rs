use std::fmt::Debug;

use objc2::rc::Retained;
use objc2_app_kit::NSPasteboard;

use crate::{ChangeWatcher, Result, catch, platform::Data};

pub struct Clipboard {
    pasteboard: Retained<NSPasteboard>,
    watcher: ChangeWatcher,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let (pasteboard, change_count) = catch(|| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let change_count = pasteboard.changeCount();
            (pasteboard, change_count)
        })?;
        Ok(Self {
            pasteboard,
            watcher: ChangeWatcher::new(change_count)?,
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
        self.watcher
            .wait_change(|| catch(|| self.pasteboard.changeCount()))
            .await
    }

    fn sync(&self) -> Result<()> {
        let change_count = catch(|| self.pasteboard.changeCount())?;
        self.watcher.sync(change_count);
        Ok(())
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}
