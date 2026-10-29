use std::ffi::OsStr;

use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};

use crate::{
    sys::{self, Error, Result},
    ui::{Data, Image},
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
    pub fn read(&mut self) -> Result<Option<Data>> {
        self.clipboard.read().map(|data| data.map(Data::new))
    }

    /// Replace the clipboard content with `data`.
    pub fn write(&mut self, data: &Data) -> Result<()> {
        self.clipboard.write(data.sys())
    }

    /// Write plain text to the clipboard.
    pub fn write_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    /// Write Rich Text Format to the clipboard.
    pub fn write_rtf(&mut self, rtf: impl AsRef<str>) -> Result<()>;

    /// Write HTML to the clipboard.
    pub fn write_html(&mut self, html: impl AsRef<str>) -> Result<()>;

    /// Write an image to the clipboard.
    pub fn write_image(&mut self, image: &Image) -> Result<()> {
        self.clipboard.write_image(&image.0)
    }

    /// Write a list of local files or URIs to the clipboard.
    pub fn write_uris(&mut self, uris: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<()>;
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
