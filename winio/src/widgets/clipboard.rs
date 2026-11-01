use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};

use crate::{
    sys::{self, Error, Result},
    ui::Data,
};

/// An accessor to the system clipboard.
#[derive(Debug)]
pub struct Clipboard {
    clipboard: sys::Clipboard,
}

#[inherit_methods(from = "self.clipboard")]
impl Clipboard {
    /// Read the current clipboard content.
    ///
    /// Returns `None` if the clipboard is empty.
    pub async fn read(&mut self) -> Result<Option<Data>> {
        self.clipboard
            .read()
            .await
            .map(|data| data.map(Data::from_sys))
    }

    /// Replace the clipboard content with `data`.
    pub async fn write(&mut self, data: &Data) -> Result<()> {
        self.clipboard.write(data.sys()).await
    }

    /// Clear the clipboard.
    pub fn clear(&mut self) -> Result<()>;
}

/// Events of [`Clipboard`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ClipboardEvent {
    /// The clipboard content changed.
    Changed,
}

/// Messages of [`Clipboard`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ClipboardMessage {}

impl Component for Clipboard {
    type Error = Error;
    type Event = ClipboardEvent;
    type Init<'a> = ();
    type Message = ClipboardMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let clipboard = sys::Clipboard::new()?;
        Ok(Self { clipboard })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.clipboard.wait_change().await;
            sender.output(ClipboardEvent::Changed);
        }
    }
}
