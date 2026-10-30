use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows::ApplicationModel::DataTransfer::Clipboard as WinClipboard;
use windows_core::EventRevoker;
use winio_callback::Callback;

use crate::{Data, GlobalRuntime, Result};

pub struct Clipboard {
    on_changed: SendWrapper<Rc<Callback>>,
    _revoker: EventRevoker,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let on_changed = SendWrapper::new(Rc::new(Callback::new()));
        let revoker = WinClipboard::ContentChanged({
            let on_changed = on_changed.clone();
            move |_, _| {
                on_changed.signal::<GlobalRuntime>(());
                Ok(())
            }
        })?;
        Ok(Self {
            on_changed,
            _revoker: revoker,
        })
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        let view = WinClipboard::GetContent()?;
        Data::from_data_package_view(&view).await
    }

    pub async fn write(&mut self, data: &Data) -> Result<()> {
        WinClipboard::SetContent(data.package())
    }

    pub fn clear(&mut self) -> Result<()> {
        WinClipboard::Clear()
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}

impl std::fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}
