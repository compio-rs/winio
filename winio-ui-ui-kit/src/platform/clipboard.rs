use std::fmt::Debug;

use objc2::{rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSArray, NSDictionary, NSString};
use objc2_ui_kit::UIPasteboard;

use crate::{ChangeWatcher, Result, catch, platform::Data};

pub struct Clipboard {
    pasteboard: Retained<UIPasteboard>,
    watcher: ChangeWatcher,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let (pasteboard, change_count) = catch(|| {
            let pasteboard = UIPasteboard::generalPasteboard();
            let change_count = unsafe { pasteboard.changeCount() };
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
        let items: Retained<NSArray<NSDictionary<NSString, AnyObject>>> =
            NSArray::from_retained_slice(&[]);
        catch(|| unsafe { self.pasteboard.setItems(&items) })?;
        self.sync()
    }

    pub async fn wait_change(&self) {
        self.watcher
            .wait_change(|| catch(|| unsafe { self.pasteboard.changeCount() }))
            .await
    }

    fn sync(&self) -> Result<()> {
        let change_count = catch(|| unsafe { self.pasteboard.changeCount() })?;
        self.watcher.sync(change_count);
        Ok(())
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}
