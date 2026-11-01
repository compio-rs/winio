use std::{cell::Cell, fmt::Debug, rc::Rc};

use gtk4::{gdk, glib::SignalHandlerId, prelude::*};
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::{Error, Result, platform::Data};

pub struct Clipboard {
    clipboard: gdk::Clipboard,
    handler: Option<SignalHandlerId>,
    inner: Rc<ClipboardInner>,
}

struct ClipboardInner {
    on_changed: Callback<()>,
    writing: Cell<bool>,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let display = gdk::Display::default().ok_or(Error::NullPointer)?;
        let clipboard = display.clipboard();
        let inner = Rc::new(ClipboardInner {
            on_changed: Callback::new(),
            writing: Cell::new(false),
        });
        let handler = clipboard.connect_changed({
            let inner = inner.clone();
            move |_| {
                if !inner.writing.get() {
                    inner.on_changed.signal::<GlobalRuntime>(());
                }
            }
        });
        Ok(Self {
            clipboard,
            handler: Some(handler),
            inner,
        })
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        Data::from_clipboard(&self.clipboard).await
    }

    pub async fn write(&mut self, data: &Data) -> Result<()> {
        let clipboard = self.clipboard.clone();
        let provider = data.content_provider()?;
        self.with_write(|| {
            match provider {
                Some(provider) => clipboard.set_content(Some(&provider))?,
                None => clipboard.set_content(None::<&gdk::ContentProvider>)?,
            }
            Ok(())
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        let clipboard = self.clipboard.clone();
        self.with_write(|| {
            clipboard.set_content(None::<&gdk::ContentProvider>)?;
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        self.inner.on_changed.wait().await
    }

    fn with_write<T>(&mut self, f: impl FnOnce() -> Result<T>) -> Result<T> {
        self.inner.writing.set(true);
        let _guard = WriteGuard(&self.inner.writing);
        f()
    }
}

struct WriteGuard<'a>(&'a Cell<bool>);

impl Drop for WriteGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        if let Some(handler) = self.handler.take() {
            self.clipboard.disconnect(handler);
        }
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}
